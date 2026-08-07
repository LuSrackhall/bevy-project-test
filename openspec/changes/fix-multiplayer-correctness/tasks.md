## 1. 测试先行(宪法 §10.1 锁步回归)

- [x] 1.1 城市捕获多人确定性测试:single-player 等价断言(0↔1 行为不变)+ 多人 FFA 归 `last_attacker_faction`(specs/city-interaction)
- [x] 1.2 PlayerSlots 槽位测试:`multi_player(9,3)` / `multi_player(16,5)` 不再 panic,返回正确槽位(specs/multiplayer-slots)
- [x] 1.3 重连重放确定性测试:重建路径(`init_simulation_world_multi` + `run_tick(enable_ai:false)`)重放命令日志后,世界 hash 与正常网络路径一致(specs/network-reconnect)

## 2. D1 解除硬编码

- [x] 2.1 `lobby_ready_mask: u8` → `HashSet<u8>`(bevy_adapter/src/network.rs:352,421-429),ready 判定 `set.len() >= active_players.len()`;适配 relay_core.rs:265-272 的读取
- [x] 2.2 `PlayerSlots::multi_player` 移除 `assert!(count <= 8)`(simulation/src/types.rs:358)
- [x] 2.3 city_capture_check_system 改为归 `last_attacker_faction`(simulation/src/soldier/mod.rs:859-864),统一任意阵营语义,保留无攻击者兜底
- [x] 2.4 `collect_command_players` 兜底 `if id <= 1` 移除(simulation/src/lib.rs:103)

## 3. D2 掉线重连

- [x] 3.1 `on_disconnect` 不再 `all_players.retain`,标记 `Disconnected` 保留席位(network.rs:595-598)
- [x] 3.2 `next_player_id` 改为分配"首个空闲席位",重连复用原 player_id(network.rs:354,408-413)
- [x] 3.3 `try_finalize` all_ready 对 `Disconnected` 席位放行,靠超时兜底定稿(R3,network.rs:506-510)
- [x] 3.4 客户端接通 `apply_reconnect`:世界重建走 `init_simulation_world_multi` + 原地图 + `run_tick(enable_ai:false)`(R1),修正 network.rs:273-274 注释;重建逻辑封装进 bevy_adapter 会话层(R5),消除 render_view/lib.rs:494 直触仿真
- [x] 3.5 `map_spec_hash` → MapSize 确定性映射,重连地图与对局一致(R4),消除 render_view/lib.rs:462 硬编码 Medium
- [x] 3.6 重连快进复用 driver Network 管线(R2):`apply_reconnect` 灌 relay_buffer → `simulation_driver_system` 消费
- [x] 3.7 重连集成测试:掉线后重连复用原 player_id / 8 人满员拒绝 / Disconnected 席位不挂起 barrier

## 4. D1b UI 参数化

- [x] 4.1 创建房间人数参数化 + cycle 到 2..=8(render_view/src/lib.rs:672 默认值、lan_lobby.rs:317,408 按钮与 ModalState)
- [x] 4.2 `current_players` 随加入/离开正确更新(render_view/src/lib.rs:699-710 + discovery/model.rs)

## 5. 合规补齐与收尾

- [x] 5.1 ADR:城市归属语义变更(0↔1 互换 → last_attacker_faction)
- [x] 5.2 ADR:重连恢复语义(席位保留 + 重建路径统一)
- [x] 5.3 全仓 `cargo test` + `cargo check` 全绿;宪法自检清单逐项过

---

## Post-Implementation Workflow

<!-- DO NOT MODIFY THIS SECTION — it defines the required workflow after all tasks are complete -->

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/<name>`

**Iteration**: If user does not accept, analyze the issue and recommend:
fix in place / new change / git reset + stash / git reset / abandon.
