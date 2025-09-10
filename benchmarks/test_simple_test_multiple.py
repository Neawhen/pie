import asyncio
import time
import argparse
from pathlib import Path
from blake3 import blake3
from pie.client import PieClient, Instance

from test_utils import append_log


async def launch_and_handle(client, args, program_hash, prompt, instance_id, priority=None):
    """启动并处理单个inferlet实例，返回完成时间信息"""

    instance_args = [
        "--prompt", prompt,
        "--max-tokens", str(args.max_tokens),
        "--model", args.model,
    ]

    # 如果启用了句号暂停模式，添加参数
    if args.semicolon_pause:
        instance_args.append("--semicolon-pause")

    # 如果启用了动态优先级更新，添加参数
    if hasattr(args, 'update_priority') and args.update_priority:
        instance_args.append("--update-priority")

    # 如果指定了静态优先级，添加参数
    if priority and priority != "dynamic":
        instance_args.extend(["--static-priority", priority])

    instance = await client.launch_instance(program_hash, arguments=instance_args)

    print(f"🚀 Starting instance {instance_id}: Prompt='{prompt}'")

    start_time = time.monotonic()
    instance_start_time = time.time()  # Record absolute start time

    while True:
        event, message = await instance.recv()
        if event == "terminated":
            end_time = time.monotonic()
            elapsed = end_time - start_time
            instance_end_time = time.time()  # Record absolute end time

            if args.verbose:
                print(f"⏱️  Instance {instance_id} completed in {elapsed:.2f} seconds")

            # Return instance completion info
            instance_info = {
                'instance_id': instance_id,
                'priority': priority or 'dynamic',
                'start_time': instance_start_time,
                'end_time': instance_end_time,
                'elapsed_seconds': elapsed,
                'prompt': prompt[:50] + '...' if len(prompt) > 50 else prompt
            }
            break
        else:
            if args.verbose:
                print(f"Instance {instance_id} received message: '{message}'")
            # Handle simple-test priority update messages
            if message.startswith("PRIORITY_UPDATE:"):
                if args.verbose:
                    print(f"📊 Instance {instance_id} priority update: {message[15:]}")
            elif message.startswith("PRIORITY_INFO:"):
                if args.verbose:
                    print(f"📈 Instance {instance_id} final priority: {message[13:]}")

    return instance_info


async def main(args):
    prompts = []
    for i in range(args.num_instances):
        # 为每个实例创建不同的提示词
        base_prompt = args.prompt or "Tell me about artificial intelligence"
        prompt = f"{base_prompt} (Instance {i+1})"
        prompts.append(prompt)

    program_name = "simple_test"
    program_path = Path(__file__).parent.parent / f"example-apps/target/wasm32-wasip2/release/{program_name}.wasm"

    if not program_path.exists():
        print(f"❌ 错误：程序文件未找到: {program_path}")
        print("请先编译 simple-test:")
        print("cd ../example-apps && cargo build --release --target wasm32-wasip2")
        return

    async with PieClient(args.server_uri) as client:
        with open(program_path, "rb") as f:
            program_bytes = f.read()
        program_hash = blake3(program_bytes).hexdigest()

        if not await client.program_exists(program_hash):
            print("📤 Program not found on server, uploading...")
            await client.upload_program(program_bytes)
            print("✅ Upload completed")

        print(f"🎯 Starting benchmark test with {args.num_instances} simple-test inferlet instances...")
        print(f"🤖 Model: {args.model}")
        print(f"🎯 Semicolon pause mode: {'enabled' if args.semicolon_pause else 'disabled'}")
        print(f"🎚️  Priority mode: {args.priority_mode}")
        if args.priority_mode == "alternate":
            print(f"   - Alternating allocation: high/low priorities")
        elif args.priority_mode == "custom":
            print(f"   - Custom mode: {args.priority_list}")
        print("-" * 60)

        start_time = time.monotonic()

        # Allocate priorities based on priority mode
        priorities = []
        if args.priority_mode == "alternate":
            # Alternating high/low priority allocation
            priority_options = ["high", "low"]
            for i in range(args.num_instances):
                priorities.append(priority_options[i % 2])
        elif args.priority_mode == "custom":
            # Use custom priority list
            if args.priority_list:
                custom_priorities = [p.strip().lower() for p in args.priority_list.split(",")]
                # Validate priority format
                valid_priorities = ['low', 'normal', 'high', 'critical', 'dynamic']
                for p in custom_priorities:
                    if p not in valid_priorities:
                        print(f"❌ Error: Invalid priority '{p}', must be: {', '.join(valid_priorities)}")
                        return
                for i in range(args.num_instances):
                    if i < len(custom_priorities):
                        priorities.append(custom_priorities[i])
                    else:
                        priorities.append("low")  # Default low priority
            else:
                priorities = ["low"] * args.num_instances
        else:
            # Default dynamic priority
            priorities = ["dynamic"] * args.num_instances

        # Display priority allocation
        print("🎯 Instance priority allocation:")
        for i, priority in enumerate(priorities):
            print(f"   Instance {i+1}: {priority}")
        print("-" * 60)

        # Create concurrent tasks, each instance with different prompt and priority
        tasks = [launch_and_handle(client, args, program_hash, prompt, i+1, priorities[i])
                for i, prompt in enumerate(prompts)]
        results = await asyncio.gather(*tasks)

        total_time = time.monotonic() - start_time
        throughput = args.num_instances / total_time if total_time > 0 else 0

        print("\n" + "=" * 60)
        print("✅ Benchmark test completed")
        print(f"📊 Total time:    {total_time * 1000:.2f} ms")
        print(f"🚀 Throughput:    {throughput:.2f} requests/second")
        print(f"🎯 Instances:     {args.num_instances}")
        print(f"🤖 Model:         {args.model}")
        print(f"🎯 Semicolon pause: {'enabled' if args.semicolon_pause else 'disabled'}")
        print(f"🔄 Priority update: {'enabled' if getattr(args, 'update_priority', False) else 'disabled'}")
        print(f"🎚️  Priority mode: {args.priority_mode}")
        print("=" * 60)

        # Display detailed completion time for each instance
        print("\n📋 Instance completion details:")
        print("-" * 80)
        print(f"{'ID':2s} | {'Priority':8s} | {'Time(ms)':12s} | {'Start':10s} | {'End':10s} | {'Time(s)':8s}")
        print("-" * 80)

        # Sort by completion time
        sorted_results = sorted(results, key=lambda x: x['end_time'])

        for result in sorted_results:
            elapsed_ms = result['elapsed_seconds'] * 1000
            start_dt = time.strftime('%H:%M:%S', time.localtime(result['start_time']))
            end_dt = time.strftime('%H:%M:%S', time.localtime(result['end_time']))

            print(f"{result['instance_id']:2d} | "
                  f"{result['priority']:8s} | "
                  f"{elapsed_ms:12.2f}ms | "
                  f"{start_dt:10s} | "
                  f"{end_dt:10s} | "
                  f"{result['elapsed_seconds']:8.2f}s")

        # Calculate statistics
        elapsed_times = [r['elapsed_seconds'] for r in results]
        min_time = min(elapsed_times)
        max_time = max(elapsed_times)
        avg_time = sum(elapsed_times) / len(elapsed_times)

        print("-" * 80)
        print(f"{'Statistics':15s} | "
              f"{min_time:10.2f}s | "
              f"{max_time:10.2f}s | "
              f"{avg_time:10.2f}s")

        print("=" * 80)

        # Log results
        instance_details = [{
            'instance_id': r['instance_id'],
            'priority': r['priority'],
            'elapsed_seconds': r['elapsed_seconds'],
            'start_time': r['start_time'],
            'end_time': r['end_time'],
            'prompt': r['prompt']
        } for r in results]

        log_data = {
            'total_time': total_time,
            'throughput': throughput,
            'model': args.model,
            'semicolon_pause': args.semicolon_pause,
            'update_priority': getattr(args, 'update_priority', False),
            'num_instances': args.num_instances,
            'priority_mode': args.priority_mode,
            'priority_list': getattr(args, 'priority_list', None),
            'priorities': priorities,
            'instance_details': instance_details,
            'statistics': {
                'min_time': min_time,
                'max_time': max_time,
                'avg_time': avg_time
            },
            'args': vars(args),
        }
        append_log('./logs/test_simple_test_multiple.json', log_data)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Simple Test Multi-instance Benchmark",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter
    )

    server_group = parser.add_argument_group('Server Configuration')
    server_group.add_argument("--server-uri", type=str, default="ws://127.0.0.1:8080",
                             help="PIE server URI")

    benchmark_group = parser.add_argument_group('Benchmark Configuration')
    benchmark_group.add_argument("--num-instances", type=int, default=10,
                                help="Total number of concurrent instances to start")
    benchmark_group.add_argument("--verbose", action="store_true",
                                help="Enable verbose output for debugging")

    wasm_args_group = parser.add_argument_group('Inferlet Parameters')
    wasm_args_group.add_argument("--prompt", type=str,
                                default="Write a short story about a robot learning to paint",
                                help="Base prompt to send to WASM program")
    wasm_args_group.add_argument("--max-tokens", type=int, default=100,
                                help="Maximum number of tokens per instance")
    wasm_args_group.add_argument("--model", type=str, default="llama-3.2",
                                help="Model name to use")
    wasm_args_group.add_argument("--semicolon-pause", action="store_true",
                               help="Enable semicolon pause mode (pause on period)")
    wasm_args_group.add_argument("--update-priority", action="store_true",
                               help="Enable dynamic priority updates (auto-increase priority per segment)")

    priority_group = parser.add_argument_group('Priority Control')
    priority_group.add_argument("--priority-mode", type=str, default="dynamic",
                               choices=["dynamic", "alternate", "custom"],
                               help="Priority allocation mode: dynamic, alternate(high/low), custom")
    priority_group.add_argument("--priority-list", type=str,
                               help="Custom priority list separated by commas. Supports: low,normal,high,critical,dynamic (e.g., 'high,dynamic,low,critical')")

    # Add priority validation function
    def validate_priority_list(priority_str):
        """Validate priority list format"""
        if not priority_str:
            return []
        priorities = [p.strip().lower() for p in priority_str.split(',')]
        valid_priorities = ['low', 'normal', 'high', 'critical']
        for p in priorities:
            if p not in valid_priorities:
                raise argparse.ArgumentTypeError(f"Invalid priority '{p}', must be: {', '.join(valid_priorities)}")
        return priorities

    args = parser.parse_args()
    asyncio.run(main(args))
