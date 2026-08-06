# ADR 0008: 城市捕获归属改为 last_attacker_faction

## 状态

**Date**: 2026-08-05
**Status**: Accepted（实现于 fix-multiplayer-correctness）

## 背景

`city_capture_check_system` 原有阵营归属逻辑硬编码 2 人语义：

```rust
let nf = match fac.0 {
    FactionId(0) => FactionId(1),           // 0 城被打 → 归 1
    FactionId(1) => FactionId(0),           // 1 城被打 → 归 0
    FactionId(2) => city.last_attacker_faction.unwrap_or(FactionId(0)),
    FactionId(_) => FactionId(2),           // 未知阵营 → 中立
};
```

在 8 人以上 FFA 下，`FactionId(2)` 被当作"中立"，且城市易手按 0↔1 互换，任意第 3 个阵营攻下的城市会被错误翻转或中立化。

## 决策

城市 HP ≤ 0 时，归属改为 **最后一个攻击者**（`last_attacker_faction`）；无攻击者记录时**保持原 owner**：

```rust
let nf = city.last_attacker_faction.unwrap_or(fac.0);
```

`last_attacker_faction` 由确定性 combat 写入（soldier/mod.rs:989）且已纳入 golden hash，是确定性纯状态。

## 理由

1. **多人正确**：任意阵营攻下城市归该阵营，符合 FFA 语义。
2. **单机等价**：单机 2 人下攻击者只能是对方，行为与旧 0↔1 互换完全一致，不破坏确定性测试。
3. **确定性**：归属完全由确定性状态推导，不引入随机/时钟。

## 影响

- 单机 2 人行为不变（等价性由 `test_capture_single_player_equivalent` 守护）。
- 多人 FFA 城市归属正确（`test_capture_multiplayer_last_attacker`）。
- 无攻击者保持 owner（`test_capture_no_attacker_keeps_owner`）。

## 关联

- specs/city-interaction（delta spec）
- 宪法 §0.1 确定性、§2.1 simulation 唯一真相源
