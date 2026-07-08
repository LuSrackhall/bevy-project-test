## Context

本变更修复 HUD 显示层 `FactionId(0)` 硬编码。详见 brainstorm-spec.md（Context/Decisions/Risks 已覆盖架构设计）。

## Decisions

### D1：lid 获取

```rust
let lid = crate::local_player_id(&*sim_world);
```

两个系统函数均已持有 `sim_world: NonSend<SimulationWorld>` 参数，零参数成本。

### D2：4 处过滤 + 1 处 match arm

- line 625：`f.0 == FactionId(0)` → `f.0 == FactionId(lid)`
- line 656：`FactionId(0) => "玩家"` → `f if *f == FactionId(lid) => "玩家"`
- line 1181：`fac.0 == FactionId(0)` → `fac.0 == FactionId(lid)`
- line 1208：`fac.0 == FactionId(0)` → `fac.0 == FactionId(lid)`
- 删除 `FactionId(1) => "敌人"` 和 `FactionId(2) => "中立"`，统一 `_ => "其他"`

## Risks

| Risk | 评级 | Mitigation |
|------|------|-----------|
| 单机兼容 | 🟢 无 | `local_player_id()` 回退 0 |
| Match arm 丢失精细标签 | 🟢 低 | 当前架构只有"玩家/其他"概念 |
