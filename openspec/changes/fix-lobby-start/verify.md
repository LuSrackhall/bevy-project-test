## Verification Summary

**Change:** fix-lobby-start

| Dimension | Status |
|-----------|--------|
| Completeness | 7/7 tasks 完成 |
| Correctness | 全部需求实现,同批丢弃场景 + 分批场景覆盖 |
| Coherence | 与 design.md 一致;3 个评审 agent 通过 |

### Key Changes
- `crates/render_view/src/lib.rs`: `lobby_update_system` Connected 阶段移除就绪 LobbyUpdate 分支后的 `return`(置 Ready 后继续迭代同批,使同批 GameStarted 转 Playing)
- `crates/render_view/tests/lobby_system.rs`(新增): 同批 `[LobbyUpdate, GameStarted]` 不丢 + 分批场景,断言 `received` 与 `State == Playing`
- `crates/bevy_adapter/tests/lobby_start_e2e.rs`(新增): 真实 transport 双客户端,自动开局 / 就绪开局两条路径送达 GameStarted

### 评审结论
- **代码评审**: APPROVE(根因核实到 relay_core 背靠背广播 + 同批 drain;回归测试真实红绿)
- **宪法合规**: COMPLIANT(仅 render_view 层,无越层/非确定性)
- **测试评审**: ADEQUATE(修复前会红的回归测试;e2e 为 transport 冒烟测试,已按建议修正注释)

### Issues
- 无 CRITICAL / WARNING
- SUGGESTION(已处理): e2e 头部注释修正为"transport 冒烟测试,非 render_view 回归";lobby_system 补 `State == Playing` 断言

### 测试
- `cargo test --workspace` 全绿(217 通过,0 失败)
