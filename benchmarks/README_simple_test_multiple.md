# Simple Test Multi-Instance Benchmark

This script `test_simple_test_multiple.py` is based on `test_5_text_completion_pie.py` and is used to launch multiple simple-test inferlet instances for concurrent benchmarking.

## Features

- ✅ **Concurrent Multi-Instance Launch**: Supports launching multiple simple-test inferlets simultaneously
- ✅ **Semicolon Pause Mode Support**: Enable/disable simple-test's unique semicolon pause functionality
- ✅ **Real-time Priority Monitoring**: Displays priority update information for each instance
- ✅ **Performance Statistics**: Records key metrics like total time, throughput, etc.
- ✅ **Flexible Configuration**: Supports custom prompts, models, maximum token counts, etc.

## Usage

### 1. Compile simple-test

First, ensure the simple-test inferlet has been compiled:

```bash
cd ../example-apps
cargo build --release --target wasm32-wasip2
```

### 2. Start PIE Server

Ensure the PIE server is running:

```bash
# Start server (assuming in another terminal)
./pie-server --port 8080
```

### 3. Run Benchmark Test

```bash
cd /home/neiwen/pie/benchmarks
python test_simple_test_multiple.py [parameters]
```

## Parameter Description

### Server Configuration
- `--server-uri`: PIE server URI (default: `ws://127.0.0.1:8080`)

### Benchmark Configuration
- `--num-instances`: Total number of concurrent instances (default: 10)
- `--verbose`: Enable verbose output for debugging

### Inferlet Parameters
- `--prompt`: Base prompt (default: "Write a short story about a robot learning to paint")
- `--max-tokens`: Maximum number of tokens to generate per instance (default: 100)
- `--model`: Model name to use (default: "llama-3.2")
- `--semicolon-pause`: Enable semicolon pause mode

## Example Usage

### Basic Usage (Launch 10 instances)
```bash
python test_simple_test_multiple.py
```

### Enable Semicolon Pause Mode
```bash
python test_simple_test_multiple.py --semicolon-pause
```

### Custom Parameters
```bash
python test_simple_test_multiple.py \
  --num-instances 20 \
  --prompt "Explain quantum computing in simple terms" \
  --max-tokens 200 \
  --model "qwen-3" \
  --semicolon-pause \
  --verbose
```

### Large-scale Concurrent Test
```bash
python test_simple_test_multiple.py \
  --num-instances 50 \
  --max-tokens 150 \
  --verbose
```

## Output Example

```
🎯 Starting benchmark test, launching 10 simple-test inferlet instances...
🤖 Model: llama-3.2
🎯 Semicolon pause mode: Disabled
------------------------------------------------------------
🚀 Launching instance 1: Prompt='Write a short story about a robot learning to paint (Instance 1)'
🚀 Launching instance 2: Prompt='Write a short story about a robot learning to paint (Instance 2)'
...
⏱️  Instance 1 completed, time: 2.45 seconds
⏱️  Instance 2 completed, time: 2.67 seconds
...

============================================================
✅ Benchmark test completed
📊 Total time:      3245.67 milliseconds
🚀 Throughput:      3.08 requests/second
🎯 Concurrent instances: 10
🤖 Model:           llama-3.2
🎯 Semicolon pause: Disabled
============================================================
```

## Differences from Original text-completion Test

| Feature | text-completion | simple-test |
|---------|----------------|-------------|
| Function | Pure text generation | Text generation with priority control |
| Pause Mechanism | None | Supports semicolon pause |
| Real-time Communication | None | Priority update messages |
| Parameter Complexity | Simple | Supports more configuration options |
| Output Monitoring | Basic | Detailed priority information |

## Logging

Test results are automatically saved to `./logs/test_simple_test_multiple.json`, containing:
- Total time
- Throughput
- Model information
- Semicolon pause mode status
- All command-line parameters

