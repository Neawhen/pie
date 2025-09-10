# Inferlet 快速测试指南

本指南将帮助你快速上手PIE项目的inferlet测试。

## 📋 目录
- [环境准备](#环境准备)
- [基础测试](#基础测试)
- [高级测试](#高级测试)
- [测试最佳实践](#测试最佳实践)
- [常见问题](#常见问题)

## 🛠️ 环境准备

### 1. 安装依赖
```bash
# 安装WebAssembly目标
rustup target add wasm32-wasip2

# 安装PIE CLI
cd pie-cli && cargo install --path .
```

### 2. 下载模型
```bash
# 下载一个测试模型
pie model add "llama-3.2-1b-instruct"
```

### 3. 启动PIE引擎
```bash
cd pie-cli
export PROTOCOL_BUFFERS_PYTHON_IMPLEMENTATION=python
export TORCH_CUDA_ARCH_LIST="8.9"
pie start --config ./example_config.toml
```

## 🚀 基础测试

### 官方基本测试
```bash
pie> run ../example-apps/target/wasm32-wasip2/release/text_completion.wasm -- --model "llama-3.2" --prompt "What is the capital of France?"
```

### 简单测试示例
我们创建了一个简单的测试示例 `simple-test`，展示基本的文本生成功能。

#### 构建测试
```bash
cd example-apps
cargo build --target wasm32-wasip2 --release
```

#### 运行测试
在PIE shell中执行：
```bash
pie> run ../example-apps/target/wasm32-wasip2/release/simple_test.wasm -- --prompt "Hello, how are you?" --max-tokens 50 --model "llama-3.2"
```

#### 运行分段测试
```bash
pie>run ../example-apps/target/wasm32-wasip2/release/simple_test.wasm -- --prompt \"Write a simple Rust function\" --max-tokens 100 --semicolon-pause
```

#### 测试功能
- ✅ 基本文本生成
- ✅ 命令行参数解析
- ✅ 模型信息获取
- ✅ 性能计时
- ✅ 结果输出

## 🎯 高级测试

### 高级测试示例
`advanced-test` 展示了更多高级功能：

#### 测试模式
1. **基础模式** (`--test-mode basic`)
   - 基本生成功能
   - 性能指标计算

2. **性能模式** (`--test-mode performance`)
   - 多次运行测试
   - 平均性能统计

3. **自定义模式** (`--test-mode custom`)
   - 自定义采样器
   - 自定义停止条件

4. **上下文模式** (`--test-mode context`)
   - 上下文分支测试
   - KV缓存管理

#### 运行高级测试
```bash
# 基础性能测试
pie> run ../example-apps/target/wasm32-wasip2/release/advanced_test.wasm -- --test-mode performance --prompt "Write a short story" --max-tokens 100

# 自定义生成测试
pie> run ../example-apps/target/wasm32-wasip2/release/advanced_test.wasm -- --test-mode custom --prompt "Explain quantum computing" --max-tokens 150

# 上下文管理测试
pie> run ../example-apps/target/wasm32-wasip2/release/advanced_test.wasm -- --test-mode context --prompt "Tell me a joke" --max-tokens 50
```

## 📊 测试最佳实践

### 1. 性能测试
```rust
// 使用Instant进行精确计时
let start_time = std::time::Instant::now();
let result = ctx.generate_until("<|eot_id|>", max_tokens).await;
let elapsed = start_time.elapsed();

// 计算性能指标
let token_count = tokenizer.tokenize(&result).len();
let tokens_per_second = token_count as f64 / elapsed.as_secs_f64();
```

### 2. 错误处理
```rust
// 使用Result类型进行错误处理
async fn test_function() -> Result<(), String> {
    let model = inferlet::get_auto_model();
    if !model.has_traits(&["input_text", "tokenize", "output_text"]) {
        return Err("模型缺少必要特性".to_string());
    }
    Ok(())
}
```

### 3. 资源管理
```rust
// Context会自动管理KV缓存
let mut ctx = model.create_context();
// 使用完毕后自动清理资源
```

### 4. 自定义采样和停止条件
```rust
use inferlet::sampler::GreedySampler;
use inferlet::stop_condition::{Until, Length, any};

let mut sampler = GreedySampler::new();
let stop_tokens = tokenizer.tokenize("The End");
let mut stop_condition = any(
    Until::new(stop_tokens),
    Length::new(max_tokens),
);
```

## 🔧 创建自定义测试

### 1. 项目结构
```
your-test/
├── Cargo.toml
└── src/
    └── lib.rs
```

### 2. Cargo.toml模板
```toml
[package]
name = "your-test"
version = "0.1.0"
edition = "2021"

[dependencies]
inferlet = "0.1.0"
pico-args = "0.5.0"
futures = "0.3.0"
```

### 3. 基本代码模板
```rust
use pico_args::Arguments;
use std::ffi::OsString;

#[inferlet::main]
async fn main() -> Result<(), String> {
    // 解析参数
    let mut args = Arguments::from_vec(
        inferlet::get_arguments()
            .into_iter()
            .map(OsString::from)
            .collect(),
    );

    // 获取模型
    let model = inferlet::get_auto_model();
    
    // 创建上下文
    let mut ctx = model.create_context();
    
    // 你的测试逻辑
    // ...
    
    Ok(())
}
```

## 🐛 常见问题

### Q: 编译错误 "target wasm32-wasip2 not found"
A: 运行 `rustup target add wasm32-wasip2`

### Q: 运行时错误 "No models available"
A: 确保已下载模型：`pie model add "llama-3.2-1b-instruct"`

### Q: 模型特性检查失败
A: 检查模型是否支持所需特性：
```rust
if !model.has_traits(&["input_text", "tokenize", "output_text"]) {
    return Err("模型缺少必要特性".to_string());
}
```

### Q: 生成结果为空
A: 检查停止条件设置，确保模型格式正确

### Q: 性能测试结果不稳定
A: 进行多次测试取平均值，考虑系统负载影响

## 📈 性能基准

### 典型性能指标
- **小模型** (1B参数): 10-50 tokens/sec
- **中等模型** (7B参数): 5-20 tokens/sec  
- **大模型** (13B+参数): 1-10 tokens/sec

### 影响因素
- 模型大小
- 硬件配置
- 系统负载
- 生成长度
- 采样策略

## 🔗 相关资源

- [PIE项目主页](https://pie-project.org)
- [示例应用集合](../example-apps/)
- [API文档](../inferlet/src/lib.rs)
- [WIT接口定义](../inferlet/wit/)

---

**提示**: 开始测试前，建议先运行简单的helloworld示例确保环境配置正确。
