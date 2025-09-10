# 🚀 PIE Simple Test 多实例基准测试 - 快速开始指南

## 前置要求

1. **安装 Rust**：
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **安装 WebAssembly target**：
   ```bash
   rustup target add wasm32-wasip2
   ```

3. **安装 Python 依赖**（可选，用于后端）：
   ```bash
   # 如果使用 Python 后端
   cd backend/backend-python
   pip install -r requirements.txt
   cd ../..
   ```

## 🔧 手动运行步骤

如果你想手动控制每个步骤：

### 1. 下载模型（如果还没有）
```bash
# 下载 llama-3.2-1b-instruct 模型
cd pie-cli
cargo run -- model add "llama-3.2-1b-instruct"
cd ..
```

### 2. 编译 Simple Test
```bash
cd example-apps
cargo build --target wasm32-wasip2 --release
cd ..
```

### 3. 启动 PIE 服务器
```bash
cd pie-cli
export PROTOCOL_BUFFERS_PYTHON_IMPLEMENTATION=python
export TORCH_CUDA_ARCH_LIST="8.9"
pie start --config ./example_config.toml
```

### 4. 运行基准测试（在新终端）
```bash
cd /home/neiwen/pie && python benchmarks/test_simple_test_multiple.py 

# 基本测试（10个实例）
python test_simple_test_multiple.py

# 启用句号暂停模式
python test_simple_test_multiple.py --semicolon-pause --verbose

# 大规模测试
python test_simple_test_multiple.py --num-instances 50 --max-tokens 200
```

## 📊 测试结果

测试完成后，结果会保存到 `logs/test_simple_test_multiple.json`，包含：
- 总用时
- 吞吐量（请求/秒）
- 模型信息
- 句号暂停模式状态

## 🛠️ 故障排除

### 常见问题

1. **编译失败**：
   ```bash
   # 清理并重新编译
   cd example-apps
   cargo clean
   cargo build --target wasm32-wasip2 --release
   ```

2. **服务器连接失败**：
   - 检查端口 8080 是否被占用
   - 确认服务器已经完全启动（需要几秒钟）

3. **模型下载失败**：
   ```bash
   # 手动清理模型缓存
   rm -rf ~/.cache/pie/models/llama-3.2-1b-instruct*
   # 重新下载
   cd pie-cli && cargo run -- model add "llama-3.2-1b-instruct"
   ```

4. **Python 依赖问题**：
   ```bash
   # 重新安装依赖
   cd backend/backend-python
   pip install --upgrade -r requirements.txt
   ```

### 日志查看

```bash
# 查看 PIE 服务器日志
tail -f pie-cli/pie.log

# 查看 Python 后端日志
tail -f ~/.cache/pie/logs/backend.log
```

## 🎛️ 高级配置

### 修改配置文件

编辑 `pie-cli/example_config.toml` 来调整：

```toml
# 服务器配置
host = "127.0.0.1"
port = 8080

# 批处理策略
batching_strategy = "adaptive"  # adaptive, k, t, kort

# 后端配置
[[backend]]
backend_type = "python"
model = "llama-3.2-1b-instruct"
device = "cuda:0"  # 或 "cpu"
```

### 自定义基准测试参数

```bash
# 查看所有可用参数
python benchmarks/test_simple_test_multiple.py --help
```

## 📈 性能优化建议

1. **使用 GPU**：确保 `device = "cuda:0"` 在配置文件中
2. **调整批处理**：尝试不同的 `batching_strategy`
3. **增加内存**：如果遇到内存不足，减少 `num-instances`
4. **监控资源**：使用 `nvidia-smi` 或 `htop` 监控系统资源

## 🔗 更多资源

- [PIE 项目主页](https://pie-project.org)
- [模型索引](https://github.com/pie-project/model-index)
- [完整文档](https://pie-project.org/docs/)

---

🎉 **现在你可以开始测试 PIE 系统的性能了！**

