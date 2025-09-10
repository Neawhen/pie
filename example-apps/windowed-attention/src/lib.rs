use inferlet::sampler::{self, Sampler};
use inferlet::stop_condition::{self, StopCondition};

use inferlet::Context;
use std::time::Instant;
use inferlet::traits::Tokenize;

/// Generates text using a simple sliding window for KV cache management.
///
/// This method keeps only the most recent `window_size` tokens in the KV cache.
/// As new tokens are generated, the oldest tokens beyond the window size are
/// masked and eventually evicted from the cache. This is the simplest form of
/// windowed attention, suitable for tasks where only recent context is relevant.

pub async fn generate_with_sliding_window<S: Sampler, C: StopCondition>(
    ctx: &mut Context,
    sampler: &mut S,
    stop_condition: &mut C,
    window_size: usize,
) -> String {
    let mut generated_token_ids = Vec::new();
    // The autoregressive generation loop
    loop {
        // 1. Decode the next token, sample, and add it to the pending buffer.
        let dist = ctx.decode_step().await;
        let next_token_id = sampler.sample(&dist.ids, &dist.probs);
        ctx.fill_token(next_token_id);
        generated_token_ids.push(next_token_id);

        // 2. Check for the stop condition.
        if stop_condition.should_stop(&generated_token_ids) {
            break;
        }

        // 3. Apply sliding window logic.
        let committed_len = ctx.token_ids.len();
        if committed_len > window_size {
            // Mask all tokens from the beginning that are now outside the window.
            let evict_end = committed_len - window_size;
            ctx.mask_token_range(1, evict_end, true);
            ctx.drop_masked_kv_pages();
        }
    }

    ctx.tokenizer.detokenize(&generated_token_ids)
}

#[inferlet::main]
async fn main() -> Result<(), String> {
    let start = Instant::now();

    let max_num_outputs = 512;
    let window_size = 32;
    let model = inferlet::get_auto_model();
    let tokenizer = model.get_tokenizer();

    let mut ctx = Context::new(&model);
    ctx.fill("<|begin_of_text|>");
    ctx.fill("<|start_header_id|>system<|end_header_id|>\n\nYou are a helpful, respectful and honest assistant.<|eot_id|>");
    ctx.fill("<|start_header_id|>user<|end_header_id|>\n\nExplain LLM decoding process in ELI5.<|eot_id|>");
    ctx.fill("<|start_header_id|>assistant<|end_header_id|>\n\n");

    let mut sampler = sampler::GreedySampler::new();

    let mut stop_condition = stop_condition::any(
        stop_condition::Until::new(tokenizer.tokenize("<|eot_id|>")),
        stop_condition::Length::new(max_num_outputs),
    );

    println!(
        "Starting generation with Windowed Attention ( window_size={})",
        window_size
    );

    // Call the standalone function
    let output =
        generate_with_sliding_window(&mut ctx, &mut sampler, &mut stop_condition, window_size)
            .await;

    let elapsed = start.elapsed();
    let output_token_ids = tokenizer.tokenize(&output);

    println!("\n--- Output ---\n{}\n--------------", output);

    println!(
        "Total elapsed: {:?}, Tokens generated: {}",
        elapsed,
        output_token_ids.len()
    );

    // compute per token latency
    if !output_token_ids.is_empty() {
        println!(
            "Per-token latency: {:?}",
            elapsed / output_token_ids.len() as u32
        );
    }

    Ok(())
}
