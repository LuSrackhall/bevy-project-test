## Context

1000+ 单位场景下存在两个卡顿问题：
1. 框选 1000+ 单位时卡顿 — `selection_visual_system` O(m*n) 每帧
2. 对 1000+ 单位下发 seek stance 指令（range > 0）后卡顿 — `combat_engagement_system` O(s*u*log u) 每 tick

根源审计发现 9 个卡顿点，归类为 4 个根因：
- 无空间索引（4 个战斗系统 O(n²)）
- 循环内排序（combat_engagement_system 每士兵排序一次）
- O(n) find_entity_by_unit_id（9 处调用）
- O(m*n) render_view 查询（selection + HUD）

已触发宪法 Tier 2 条件（实体 > 1000）。

## Goals / Non-Goals

**Goals:**
- 消除所有 O(n²) 战斗热路径
- 将 find_entity_by_unit_id 从 O(n) 降为 O(1)
- 将 selection_visual_system 和 HUD 查询从 O(m*n) 降为 O(m)
- 保持 §2.6 确定性
- 修复 2 个已有 bug

**Non-Goals:**
- 不修改宪法
- 不改变游戏逻辑/行为
- 不引入新 crate 依赖

## Decisions

### Decision 1: UnitIdEntityIndex（每 tick 重建）

simulation 层新增 `UnitIdEntityIndex(HashMap<UnitId, Entity>)` Resource。**每 tick 开头从全量实体重建**，不维护增量更新。

**理由**：
- 与 bevy_adapter 的 UnitIdMapper 不冲突——前者是 simulation 内部派生数据（类似 hash_world_state），后者是跨层桥接
- 零维护成本，不会出现悬空映射
- 不违反 §17 Truth Ownership（派生数据无归属冲突）
- O(n) 重建成本远低于被替换的 O(n²) 热点
- 无需 ADR（未改变模块职责边界）

**替代方案**：将 UnitIdMapper 从 bevy_adapter 下沉到 simulation — 违反 §17.2 归属矩阵，放弃。

### Decision 2: SpatialHash 改造（BTreeMap + cell 内排序）

将 SpatialHash 内部的 `HashMap<(i32,i32), Vec<...>>` 改为 `BTreeMap`，cell 内 Vec 按 UnitId 排序。

**理由**：
- HashMap 迭代顺序不确定（Rust SipHash 随机化种子），战斗系统中等距敌人的选择会因遍历顺序不同而分歧，违反 §2.6
- BTreeMap 按 key 字典序遍历，完全确定
- cell 内按 UnitId 排序保证同一 cell 内遍历顺序确定
- SpatialHash 需要在 cell 中存储 UnitId（当前只存 FixedVec2 + u32），改造时一并加入

**实现注意点**：SpatialHash 的 `insert` 和 `query_nearby` 签名需要扩展，加入 UnitId 参数。

### Decision 3: 4 个战斗系统推广 SpatialHash

| 系统 | 当前复杂度 | 优化后 |
|------|----------|--------|
| combat_engagement_system | O(s * u * log u) | O(s * k)，k=邻域大小 |
| melee_attack_system | O(a * u) | O(a * k) |
| archer_attack_system | O(archers * u) | O(archers * k) |
| arrow_movement_system | O(arrows * soldiers) | O(arrows * k) |

额外：combat_engagement_system 的 sorted_ids 排序从循环内移到循环外。

### Decision 4: render_view 层查询优化

- selection_visual_system：用 UnitIdEntityIndex 查 Entity，再 get LogicalPosition，O(m)
- HUD 多处 find_entity_by_unit_id：改用 UnitIdEntityIndex O(1) 查找

### Decision 5: 修复 2 个已有 Bug

1. **city_interaction_system**：UnitDestroyed 事件中 unit_id 硬编码为 UnitId(0) → despawn 前获取实际 UnitId
2. **arrow_movement_system**：箭矢衰减销毁时未发出 UnitDestroyed 事件 → 补发事件

## Risks / Trade-offs

**[Risk] UnitIdEntityIndex 每 tick 重建 O(n) 成本** → n=1000 时约 0.1ms，远低于被替换的 O(n²) 成本

**[Risk] SpatialHash BTreeMap 比 HashMap 慢** → cell 数 < 1000 时差异可忽略，确定性收益远大于性能成本

**[Risk] 4 个战斗系统改造影响 40+ 测试** → 只改查询方式不改逻辑，测试行为不变

**[Risk] Bug 修复改变事件内容** → hash_world_state 不覆盖 SimulationEvents，不影响 golden_test

## Implementation Order

1. SpatialHash 改造（BTreeMap + UnitId + cell 内排序）
2. UnitIdEntityIndex（每 tick 重建）
3. 4 个战斗系统推广 SpatialHash + sorted_ids 外移
4. 修复 2 个 Bug（city_interaction + arrow_decay）
5. selection_visual_system HashMap 优化
6. HUD 查询优化
7. 全量测试验证
