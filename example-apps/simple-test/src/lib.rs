use pico_args::Arguments;
use std::ffi::OsString;
use inferlet::sampler::{self, Sampler};


fn string_to_priority(priority_str: &str) -> usize {
    match priority_str.to_lowercase().as_str() {
        "low" => 0,
        "normal" => 1,
        "high" => 2,
        "critical" => 3,
        _ => 0,
    }
}

/// 字符串转换为 inferlet Priority 枚举
fn string_to_inferlet_priority(priority_str: &str) -> inferlet::Priority {
    match priority_str.to_lowercase().as_str() {
        "low" => inferlet::Priority::Low,
        "normal" => inferlet::Priority::Normal,
        "high" => inferlet::Priority::High,
        "critical" => inferlet::Priority::High, 
        _ => inferlet::Priority::Normal, 
    }
}

/// 字符串优先级转换为下一个优先级（用于动态优先级提升）
fn next_priority_string(current_priority: &str) -> String {
    match current_priority.to_lowercase().as_str() {
        "low" => "normal".to_string(),
        "normal" => "high".to_string(),
        "high" => "critical".to_string(),
        "critical" => "critical".to_string(), 
        _ => "normal".to_string(), 
    }
}

/// 带句号暂停的文本生成函数
/// 当遇到句号时暂停生成，可选择是否同时更新优先级
/// 返回 (生成的文本, 最终优先级字符串, 总token数)
async fn generate_with_period_pause(
    ctx: &mut inferlet::Context,
    queue: &inferlet::Queue,
    max_tokens: usize,
    initial_priority_str: &str,
    update_priority: bool,
) -> Result<(String, String, usize), String> {
    let mut sampler = sampler::GreedySampler::new();
    let mut tokens_generated = 0;
    let initial_text_len = ctx.get_text().len(); // Record initial text length
    let mut last_pause_len = initial_text_len; // Record text length at last pause
    let mut current_priority_str = initial_priority_str.to_string(); // Current priority string

    let mode_desc = if update_priority {
        "pausing and increasing priority on '.'"
    } else {
        "pausing but not changing priority on '.'"
    };
    println!("🎯 Starting incremental generation mode, {}...", mode_desc);
    println!("📊 Initial state: Priority {} | Generated tokens: {}", current_priority_str, tokens_generated);

    // Set initial priority
    let initial_priority = string_to_inferlet_priority(&current_priority_str);
    queue.set_priority(initial_priority);

    loop {
        // Perform single-step generation
        let dist = ctx.decode_step().await;

        // Sample next token
        let next_token_id = sampler.sample(&dist.ids, &dist.probs);
        ctx.fill_token(next_token_id);
        tokens_generated += 1;

        // Get current complete generated text (including prompt)
        let current_text = ctx.get_text();

        // Extract newly generated text portion (excluding initial prompt)
        let _generated_text = if current_text.len() > initial_text_len {
            &current_text[initial_text_len..]
        } else {
            ""
        };

        // Calculate newly added content this time
        let new_content = if current_text.len() > last_pause_len {
            &current_text[last_pause_len..]
        } else {
            ""
        };

        // Check if new content contains period
        if new_content.contains('.') {
            let old_priority = current_priority_str.clone();

            // Update priority based on settings
            if update_priority {
                let new_priority_str = next_priority_string(&current_priority_str);
                current_priority_str = new_priority_str.clone();
                let new_priority = string_to_inferlet_priority(&current_priority_str);
                queue.set_priority(new_priority);
            }

            println!("\n⏸️  Period detected, pausing generation");
            println!("📝 New content: {}", new_content.trim());

            if update_priority {
                println!("🎯 Generation priority: {} → {} | Generated tokens: {}", old_priority, current_priority_str, tokens_generated);
            } else {
                println!("🎯 Generation priority: {} (unchanged) | Generated tokens: {}", current_priority_str, tokens_generated);
            }

            // Real-time priority update is now handled by set_priority calls above

            // Update last pause position
            last_pause_len = current_text.len();
            println!("▶️  Continuing generation...");
        }

        // Check if maximum token count is reached
        if tokens_generated >= max_tokens {
            println!("\n🏁 Maximum token limit reached ({})", max_tokens);
            println!("📊 Final statistics: Priority {} | Total tokens {}", current_priority_str, tokens_generated);
            break;
        }
    }

    // Return final generated text and statistics (excluding initial prompt)
    let final_text = ctx.get_text();
    let generated_text = if final_text.len() > initial_text_len {
        final_text[initial_text_len..].to_string()
    } else {
        String::new()
    };

    Ok((generated_text, current_priority_str, tokens_generated))
}

/// 动态优先级文本生成函数
/// 基于内容长度自动更新优先级，不分段暂停
/// 返回 (生成的文本, 最终优先级字符串, 总token数)
async fn generate_with_dynamic_priority(
    ctx: &mut inferlet::Context,
    queue: &inferlet::Queue,
    max_tokens: usize,
    initial_priority_str: &str,
) -> Result<(String, String, usize), String> {
    let mut sampler = sampler::GreedySampler::new();
    let mut tokens_generated = 0;
    let initial_text_len = ctx.get_text().len(); // Record initial text length
    let mut current_priority_str = initial_priority_str.to_string(); // Current priority string
    let mut last_update_tokens = 0; // Token count at last priority update

    println!("🎯 Starting dynamic priority generation mode...");
    println!("📊 Initial state: Priority {} | Generated tokens: {}", current_priority_str, tokens_generated);

    // Set initial priority
    let initial_priority = string_to_inferlet_priority(&current_priority_str);
    queue.set_priority(initial_priority);

    loop {
        // Perform single-step generation
        let dist = ctx.decode_step().await;

        // Sample next token
        let next_token_id = sampler.sample(&dist.ids, &dist.probs);
        ctx.fill_token(next_token_id);
        tokens_generated += 1;

        // Check if priority needs to be updated every 50 tokens
        if tokens_generated - last_update_tokens >= 50 && tokens_generated > 0 {
            let old_priority = current_priority_str.clone();
            let new_priority_str = next_priority_string(&current_priority_str);
            current_priority_str = new_priority_str.clone();
            last_update_tokens = tokens_generated;

            println!("🔄 Token {}: Priority {} → {}", tokens_generated, old_priority, current_priority_str);

            // Update priority using set_priority
            let new_priority = string_to_inferlet_priority(&current_priority_str);
            queue.set_priority(new_priority);
        }

        // Check if maximum token count is reached
        if tokens_generated >= max_tokens {
            println!("\n🏁 Maximum token limit reached ({})", max_tokens);
            println!("📊 Final statistics: Priority {} | Total tokens {}", current_priority_str, tokens_generated);
            break;
        }
    }

    // Return final generated text and statistics (excluding initial prompt)
    let final_text = ctx.get_text();
    let generated_text = if final_text.len() > initial_text_len {
        final_text[initial_text_len..].to_string()
    } else {
        String::new()
    };

    Ok((generated_text, current_priority_str, tokens_generated))
}

/// 静态优先级文本生成函数
/// 使用固定的优先级进行生成，不会在运行时改变
/// 返回 (生成的文本, 优先级字符串, 总token数)
async fn generate_with_static_priority(
    ctx: &mut inferlet::Context,
    queue: &inferlet::Queue,
    max_tokens: usize,
    static_priority_str: &str,
) -> Result<(String, String, usize), String> {
    let mut sampler = sampler::GreedySampler::new();
    let mut tokens_generated = 0;
    let initial_text_len = ctx.get_text().len(); // Record initial text length

    println!("🎯 Starting static priority generation mode...");
    println!("📊 Fixed priority: {} | Target tokens: {}", static_priority_str, max_tokens);

    // Set initial priority
    let initial_priority = string_to_inferlet_priority(static_priority_str);
    queue.set_priority(initial_priority);

    loop {
        // Perform single-step generation
        let dist = ctx.decode_step().await;

        // Sample next token
        let next_token_id = sampler.sample(&dist.ids, &dist.probs);
        ctx.fill_token(next_token_id);
        tokens_generated += 1;

        // Check if maximum token count is reached
        if tokens_generated >= max_tokens {
            println!("\n🏁 达到最大token数限制 ({})", max_tokens);
            println!("📊 最终统计: 优先级 {} | 总token数 {}", static_priority_str, tokens_generated);
            break;
        }
    }

    // Return final generated text and statistics (excluding initial prompt)
    let final_text = ctx.get_text();
    let generated_text = if final_text.len() > initial_text_len {
        final_text[initial_text_len..].to_string()
    } else {
        String::new()
    };

    Ok((generated_text, static_priority_str.to_string(), tokens_generated))
}

/// 简单的inferlet测试示例
/// 展示基本的文本生成功能，支持句号暂停和静态优先级
#[inferlet::main]
async fn main() -> Result<(), String> {
    // 1. 获取命令行参数
    let mut args = Arguments::from_vec(
        inferlet::get_arguments()
            .into_iter()
            .map(OsString::from)
            .collect(),
    );

    // 2. 解析参数
    let prompt = args
        .opt_value_from_str(["-p", "--prompt"])
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "Hello, how are you?".to_string());

    let max_tokens: u32 = args
        .opt_value_from_str(["-n", "--max-tokens"])
        .map_err(|e| e.to_string())?
        .unwrap_or(50);

    let model_name: String = args
        .opt_value_from_str(["-m", "--model"])
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "llama-3.2".to_string());

    let use_semicolon_pause: bool = args.contains(["-s", "--semicolon-pause"]);

    let use_update_priority: bool = args.contains(["-u", "--update-priority"]);

    // 新增：静态优先级参数
    let static_priority_opt: Option<String> = args
        .opt_value_from_str(["-r", "--static-priority"])
        .map_err(|e| e.to_string())?;

    let static_priority_str = if let Some(priority_str) = static_priority_opt {
        println!("🎯 收到静态优先级参数: {}", priority_str);
        priority_str.clone()
    } else {
        println!("🎯 使用默认优先级: low");
        "low".to_string() // Default low priority
    };

    // 3. Get model and create context
    println!("🚀 Starting simple test inferlet...");
    println!("📝 Prompt: {}", prompt);
    println!("🔢 Max tokens: {}", max_tokens);
    println!("🤖 Model: {}", model_name);
    println!("🎯 Semicolon pause mode: {}", if use_semicolon_pause { "enabled" } else { "disabled" });
    println!("🔄 Dynamic priority update: {}", if use_update_priority { "enabled" } else { "disabled" });
    println!("🎚️  Initial priority: {}", static_priority_str);

    let model = inferlet::get_auto_model();
    println!("✅ Successfully loaded model: {}", model.get_name());

    // 4. Create inference context and queue
    let mut ctx = model.create_context();
    let queue = model.create_queue();

    // 5. Set prompt format based on model type
    if model_name == "llama-3.2" {
        ctx.fill("<|begin_of_text|>");
        ctx.fill("<|start_header_id|>user<|end_header_id|>\n\n");
        ctx.fill(&prompt);
        ctx.fill("<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n");
    } else if model_name == "qwen-3" {
        ctx.fill("<|im_start|>user\n");
        ctx.fill(&prompt);
        ctx.fill("<|im_end|>\n<|im_start|>assistant\n");
    } else {
        // Default format
        ctx.fill(&prompt);
    }

    // 6. Generate text
    println!("⚡ Starting text generation...");
    let start_time = std::time::Instant::now();

    // 🔧 修复：设置初始静态优先级
    let initial_priority_str = match static_priority_str.as_str() {
        "critical" => "high", // 映射critical到high，因为inferlet只支持low/normal/high
        "high" => "high",
        "normal" => "normal",
        "low" => "low",
        _ => "normal", // 默认normal
    };
    let initial_priority = string_to_inferlet_priority(initial_priority_str);
    queue.set_priority(initial_priority);

    let (generated_text, final_priority, total_tokens) = match (use_semicolon_pause, use_update_priority) {
        (true, true) => {
            // Semicolon pause + dynamic priority update
            println!("🎛️  Mode: Segmented pause + dynamic priority");
            generate_with_period_pause(&mut ctx, &queue, max_tokens as usize, &static_priority_str, true).await?
        },
        (true, false) => {
            // Semicolon pause only, no priority update
            println!("🎛️  Mode: Segmented pause (priority unchanged)");
            generate_with_period_pause(&mut ctx, &queue, max_tokens as usize, &static_priority_str, false).await?
        },
        (false, true) => {
            // Dynamic priority update only, no segmentation
            println!("🎛️  Mode: Dynamic priority (no segmentation)");
            generate_with_dynamic_priority(&mut ctx, &queue, max_tokens as usize, &static_priority_str).await?
        },
        (false, false) => {
            // Standard mode
            if static_priority_str != "low" {
                // Use static priority mode
                println!("🎛️  Mode: Static priority");
                generate_with_static_priority(&mut ctx, &queue, max_tokens as usize, &static_priority_str).await?
            } else {
                // Use standard generation mode
                println!("🎛️  Mode: Standard generation");
                let text = ctx.generate_until("<|eot_id|>", max_tokens as usize).await;
                (text, static_priority_str, max_tokens as usize)
            }
        }
    };

    let elapsed = start_time.elapsed();
    println!("⏱️  Generation time: {:?}", elapsed);

    // 7. Output results
    println!("📤 Generated result:");

    // If semicolon pause mode is enabled, clean up output format
    if use_semicolon_pause {
        // Remove possible extra token markers, but keep actual generated content
        let temp_text = generated_text
            .replace("<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n", "")
            .replace("<|eot_id|>", "");
        let cleaned_text = temp_text.trim();

        if !cleaned_text.is_empty() {
            println!("{}", cleaned_text);
        } else {
            println!("(No content generated)");
        }
    } else {
        println!("{}", generated_text);
    }

    // 8. Send results and priority info back to controller
    // Send final generated text
    inferlet::send(&generated_text);

    // Send priority info (in JSON format)
    let priority_info = format!("{{\"priority\": \"{}\", \"total_tokens\": {}, \"sentences\": {}}}", final_priority, total_tokens, string_to_priority(&final_priority));
    inferlet::send(&format!("PRIORITY_INFO:{}", priority_info));

    println!("✅ Test completed! Final priority: {}", final_priority);
    Ok(())
}
