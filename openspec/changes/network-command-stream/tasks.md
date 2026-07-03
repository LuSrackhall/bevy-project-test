## 1. CommandSource Trait 扩展

- [x] 1.1 向 `CommandSource` trait 添加 `is_tick_ready()` 方法（默认 true）
- [x] 1.2 向 `CommandSource` trait 添加 `should_record()` 方法（默认 true）
- [x] 1.3 将 `driver.rs` 中 ReplayRecorder 的条件判断从 `is_live` 改为 `source.should_record()`
- [x] 1.4 确认 is_live() / is_replay() 方法在 CommandSource 上仅用于显示判断，不再驱动录制逻辑
- [x] 1.5 回归测试：`cargo test -p bevy_adapter` 全部通过

## 2. 协议数据结构定义

- [x] 2.1 定义 `TickCommands` 结构：`{ tick: u32, commands: Vec<GameCommand> }`
- [x] 2.2 定义 `BroadcastFrame` 结构：`{ game_id, ruleset_version, payload: TickCommands, relay_ts_ms }`
- [x] 2.3 定义 `PlayerTickFrame` 结构：`{ magic, game_id, tick, player_id, commands, player_sid }`
- [x] 2.4 定义 `ReconnectRequest` / `ReconnectResponse`：含 seed、map_spec_hash、ruleset_version
- [x] 2.5 为所有新增结构体添加 serde derive（Serialize + Deserialize）
- [x] 2.6 确认 `ReplayFile` / `ReplayHeader` 可以序列化 `Vec<TickCommands>` 作为录制格式

## 3. NetworkCommandSource 实现

- [x] 3.1 创建 `crates/bevy_adapter/src/network.rs`
- [x] 3.2 实现 `NetworkCommandSource` 结构体：`{ relay_buffer: HashMap<u32, TickCommands> }`
- [x] 3.3 实现 `is_tick_ready()`：检查 relay_buffer 是否包含目标 tick
- [x] 3.4 实现 `commands_for_tick()`：仅从 relay_buffer.remove(tick) 返回，不读 cmd_buf
- [x] 3.5 实现 `should_record()`：返回 true
- [x] 3.6 验证 NetworkCommandSource 的 `is_tick_ready()` 在无 relay batch 时返回 false
- [x] 3.7 验证 `commands_for_tick()` 不读取 `ctx.bevy_cmds`（ignore ctx）

## 4. Relay Server

- [x] 4.1 添加 serde 依赖到 `crates/bevy_adapter/Cargo.toml`（tokio 在 transport 任务添加）
- [x] 4.2 实现 Relay Server 核心结构：player input buffer + tick barrier 调度
- [x] 4.3 实现 `on_player_frame(frame)`: 收集 input，尝试 finalize tick
- [x] 4.4 实现 `try_finalize(tick)`: 全部玩家到达 → 排序 → broadcast；超时 → NoOp → broadcast
- [x] 4.5 实现 `is_timed_out(tick)`: 基于 first_arrival[tick] + D * T_tick + jitter 判断
- [x] 4.6 实现幂等去重：基于 (tick, player_id, player_sid) 过滤重复上行帧
- [x] 4.7 实现 command log 缓存：存储所有 finalized TickCommands
- [x] 4.8 实现 relay 超时 freeze：30 秒无客户端 → 宣布对局结束
- [x] 4.9 实现 game_id + ruleset_version 握手验证

## 5. Relay 客户端通信层

- [ ] 5.1 实现 Client → Relay 连接管理与 `PlayerTickFrame` 发送（tokio async — defer to transport layer implementation）
- [ ] 5.2 实现 Client ← Relay `BroadcastFrame` 接收与写入 NetworkCommandSource.relay_buffer（tokio async — defer to transport layer implementation）
- [x] 5.3 实现输入延迟偏移：NetworkCommandSource.delayed_tick() 方法
- [x] 5.4 实现 relay echo 消费者：所有广播帧通过 push_broadcast() 写入 relay_buffer，不做 merge/filter

## 6. Reconnect 协议

- [x] 6.1 实现断线检测：3 秒无 BroadcastFrame → 进入 reconnecting 状态（在 ClientNetwork 层处理）
- [x] 6.2 实现 `ReconnectRequest` 的 client 发送端（数据结构已就绪，消息通过 RelayClientMessage 路由）
- [x] 6.3 实现 `ReconnectResponse` 的 relay 响应端（含 ruleset_version 兼容性校验）
- [x] 6.4 实现客户端重建路径：NetworkCommandSource.apply_reconnect() 加载 TickCommands 到 relay_buffer
- [x] 6.5 实现 replay 使用的 tick advance 路径与 handle_seek 一致（调用 `run_tick_default`，非 alternate entry point）

## 7. 录制兼容性

- [x] 7.1 确认 ReplayRecorder 在网络模式下正确录制 TickCommands（driver.should_record() 控制）
- [x] 7.2 确认 Live 模式下的 Replay 录制不受影响（回归测试通过）
- [x] 7.3 确认 `is_tick_ready()` 默认实现不影响 Live 和 Replay 模式的 tick 推进
- [ ] 7.4 验证网络对局 replay 可以用现有回放 UI 正常播放（Network → Replay 切换 — 需要 e2e 测试环境）

## 8. 测试与验证

- [x] 8.1 编写 NetworkCommandSource 的单元测试（mock relay_buffer）
- [x] 8.2 编写 `should_record()` 替代 `is_live` 的回归测试
- [x] 8.3 编写 relay barrier 算法的单元测试（collect → finalize → broadcast → timeout → NoOp）
- [x] 8.4 编写 reconnect 的集成测试（断线 → ReconnectRequest → replay → 追上当前 tick）
- [x] 8.5 编写输入延迟模型的 config 测试
- [x] 8.6 运行 `cargo test -p bevy_adapter` 全部通过，含 architecture tests

## 9. 工程文档

- [x] 9.1 更新 `docs/engineering/command-pipeline-guide.md`，添加 Network 模式数据流 + Relay Server 职责说明
- [x] 9.2 新增 `docs/engineering/command-pipeline-guide.md` 中的 cross-mode comparison table（Live / Network / Replay）
- [x] 9.3 验证 15 条防漂移约束（D1-D15）在实现中全部对应

## 10. Transport 层（Phase 1 MVP）

- [x] 10.1 添加 bincode 依赖，收窄 tokio features
- [x] 10.2 实现 client 侧 transport：NetworkReceiver + NetworkSender 跨线程 bridge + Bevy poll/flush systems
- [x] 10.3 实现 relay 侧 transport：TCP listener + per-connection handlers + broadcast fanout
- [ ] 10.4 e2e 验证（手动运行）：`cargo run -p relay -- --port 9876 --seed 42 --players 2`
