# PIE String Priority Control System

This document describes how to implement fine-grained priority control in the PIE system, **using string priorities exclusively**, without relying on numbers. The system supports specifying that certain inferlet instances always maintain high priority while others remain low priority.

## 🎯 Core Features

- **Pure String Priorities**: Input, processing, and output all use strings (`low`, `normal`, `high`, `critical`)
- **Completely Decoupled Features**: Segmentation pause and priority updates can be controlled independently
- **Static Priority Control**: Specify fixed priority when instance starts, unchanged during runtime
- **Dynamic Priority Boost**: Automatic priority increase based on content or token count
- **Flexible Combinations**: Support for 4 different operation modes
- **Backward Compatible**: Maintains compatibility with existing systems

## Feature Characteristics

### Decoupled Control Parameters

- **`--semicolon-pause`**: Only enables segmentation pause functionality (pause on period)
- **`--update-priority`**: Only enables dynamic priority updates (auto-increase priority)
- **Combined Usage**: Both parameters can be used separately or together

### Operation Modes

The system supports 4 different operation modes:

1. **Standard Generation**: Basic text generation, no special features
2. **Segmentation Only**: Pause on period, but priority unchanged
3. **Dynamic Priority Only**: Auto-increase priority every 50 tokens, no segmentation
4. **Segmentation + Dynamic Priority**: Pause on period and increase priority (original functionality)

### Priority System

- **Static Priority**: Specify fixed priority when instance starts, unchanged during runtime
- **Dynamic Priority**: Instance priority adjusts dynamically based on content or token count
- **Mixed Mode**: Same system can run both static and dynamic priority instances simultaneously

## Supported Priority Levels

- `low`: Low priority
- `normal`: Normal priority
- `high`: High priority
- `critical`: Critical priority

## Usage Instructions

### 1. Compile Application

First compile the modified simple-test application:

```bash
cd example-apps
cargo build --release --target wasm32-wasip2
```

### 2. Validate Changes

Run the validation script to ensure all modifications are correct:

```bash
python validate_priority_system.py
```

### 3. Decoupled Feature Usage Examples

#### Segmentation Only (Priority Unchanged)
```bash
pie> run example-apps/target/wasm32-wasip2/release/simple_test.wasm -- --prompt "Task" --max-tokens 100 --semicolon-pause
```

#### Dynamic Priority Only (No Segmentation)
```bash
pie> run example-apps/target/wasm32-wasip2/release/simple_test.wasm -- --prompt "Task" --max-tokens 100 --update-priority
```

#### Segmentation + Dynamic Priority Update
```bash
pie> run example-apps/target/wasm32-wasip2/release/simple_test.wasm -- --prompt "Task" --max-tokens 100 --semicolon-pause --update-priority
```

### 4. Launch Single Static Priority Instance

#### High Priority Instance
```bash
pie> run example-apps/target/wasm32-wasip2/release/simple_test.wasm -- --prompt "High priority task" --max-tokens 100 --static-priority high
```

#### Low Priority Instance
```bash
pie> run example-apps/target/wasm32-wasip2/release/simple_test.wasm -- --prompt "Low priority task" --max-tokens 100 --static-priority low
```

#### 动态优先级实例（原有功能）
```bash
pie> run example-apps/target/wasm32-wasip2/release/simple_test.wasm -- --prompt "Dynamic task" --max-tokens 100 --semicolon-pause
```

### 5. Launch Multi-instance Benchmark Tests

#### Alternating High/Low Priority
```bash
python benchmarks/test_simple_test_multiple.py --num-instances 4 --priority-mode alternate
```

#### Custom Priority Allocation
```bash
python benchmarks/test_simple_test_multiple.py --num-instances 4 --priority-mode custom --priority-list "high,low,critical,low"
```

#### 仅分段暂停（优先级不变）
```bash
python benchmarks/test_simple_test_multiple.py --num-instances 4 --semicolon-pause
```

#### 仅动态优先级更新（不分段）
```bash
python benchmarks/test_simple_test_multiple.py --num-instances 4 --update-priority
```

#### 分段暂停 + 动态优先级更新
```bash
python benchmarks/test_simple_test_multiple.py --num-instances 4 --semicolon-pause --update-priority
```

#### 自定义优先级 + 仅分段暂停
```bash
python benchmarks/test_simple_test_multiple.py \
  --num-instances 4 \
  --priority-mode custom \
  --priority-list "high,low,critical,normal" \
  --semicolon-pause
```

### 4. 运行演示脚本

使用提供的演示脚本快速体验静态优先级控制：

```bash
python test_priority_control.py
```

## Command Line Parameters

### simple-test Application Parameters

- `--static-priority {low|normal|high|critical}`: Set static priority
- `--semicolon-pause`: Enable segmentation pause mode (pause on period)
- `--update-priority`: Enable dynamic priority updates (auto-increase priority)
- `--prompt`: Input prompt text
- `--max-tokens`: Maximum number of tokens to generate
- `--model`: Model name to use

### Benchmark Script Parameters

- `--priority-mode {dynamic|alternate|custom}`: Priority allocation mode
  - `dynamic`: All instances use dynamic priority
  - `alternate`: Alternating high/low priority allocation
  - `custom`: Use custom priority list
- `--priority-list`: Custom priority list separated by commas
- `--semicolon-pause`: Enable segmentation pause mode (all instances)
- `--update-priority`: Enable dynamic priority updates (all instances)

## How It Works

### Static Priority Mode

1. Instance priority is specified via `--static-priority` parameter at startup
2. Instance maintains the same priority throughout its entire runtime
3. Priority information is sent to the controller in real-time via `PRIORITY_UPDATE` messages
4. Controller can make scheduling decisions based on instance static priorities

### Dynamic Priority Mode (Original Functionality)

1. 实例启动时启用 `--semicolon-pause` 模式
2. 遇到句号时自动暂停并提升优先级
3. 优先级在运行时根据内容动态调整
4. 适用于需要根据生成内容调整优先级的场景

## 优先级控制器

使用 `priority_controller.py` 可以实现更复杂的优先级管理：

```python
from priority_controller import PriorityController

async with PriorityController() as controller:
    # 启动优先级监控器
    await controller.run_priority_monitor()
```

## 配置文件

创建 `instance_priority_config.json` 文件来配置实例优先级映射：

```json
{
  "description": "PIE实例优先级配置文件",
  "instance_priorities": {
    "high_priority_instance_1": "ALWAYS_HIGH",
    "low_priority_instance_1": "ALWAYS_LOW",
    "dynamic_instance_1": "DYNAMIC"
  }
}
```

## 观察效果

运行静态优先级控制时，你应该观察到：

1. **高优先级实例**获得更多计算资源，生成速度更快
2. **低优先级实例**可能被延迟处理，生成速度较慢
3. **动态优先级实例**根据内容动态调整优先级
4. 系统整体吞吐量和资源利用率得到优化

## 日志和监控

所有优先级信息都会通过标准消息机制发送，可以通过：

1. 控制器监听 `PRIORITY_UPDATE` 和 `PRIORITY_INFO` 消息
2. 查看基准测试生成的日志文件
3. 观察控制台输出中的优先级信息

## 扩展应用

这个静态优先级控制机制可以扩展到：

1. **服务质量(QoS)管理**: 为不同用户/任务分配不同优先级
2. **资源调度优化**: 根据业务重要性动态调整资源分配
3. **负载均衡**: 在多个实例间智能分配工作负载
4. **实时系统**: 确保关键任务获得及时响应

## 故障排除

### 常见问题

1. **编译失败**: 确保使用正确的Rust版本和wasm32-wasip2目标
2. **实例启动失败**: 检查程序文件路径和参数格式
3. **优先级不生效**: 确认使用了 `--static-priority` 参数
4. **消息监听失败**: 检查控制器是否正确启动

### 调试技巧

1. 启用详细输出查看优先级消息
2. 使用 `--verbose` 参数查看基准测试详情
3. 检查日志文件中的优先级分配信息
4. 观察控制台输出的优先级更新消息
