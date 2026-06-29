## Context

上一轮性能优化（perf-combat-selection-optimization）已解决 find_entity_by_unit_id O(1)、selection_visual_system O(m)、HUD 查询优化。但 4 个战斗系统仍是 O(n²) 全量扫描，是 1000+ 单位场景下的主要瓶颈。

上次尝试将 combat_engagement_system 改用 SpatialHash 失败，根因是 SpatialHash 查询循环内做 faction 检查时访问 World 导致借用冲突。

## Goals / Non-Goals

**Goals:**
- 4 个战斗系统全部改用 SpatialHash，消除 O(n²) 热路径
- 预构建 faction_map 解决借用冲突
- 保持 §2.6 确定性
- 19 个现有测试不需调整

**Non-Goals:**
- 不改游戏逻辑
- 不改宪法
- 不引入新 crate

## Decisions

### Decision 1: 每个系统独立 SpatialHash（不同 cell_size）

不同系统查询范围差异大，共用一个 SpatialHash 会导致小范围系统收到过多无关实体。

| 系统 | 查询范围 | cell_size | 3x3 覆盖 |
|------|---------|-----------|---------|
| combat_engagement | seek_range (~150) | 64 | 192 |
| melee_attack | 30 | 32 | 96 |
| archer_attack | 380-600 | 200 | 600 |
| arrow_movement | 碰撞半径 22 | 32 | 96 |

### Decision 2: 预构建 faction_map 避免借用冲突

从已收集的 all_units HashMap 拆出 `HashMap<UnitId, Faction>`。SpatialHash 循环内只读 faction_map，不碰 World。

箭矢系统额外需要区分 soldier 和 city 的 faction，分别构建两个 map。

### Decision 3: combat_engagement_system 的 seek_range 处理

seek_range 是运行时组件值（SeekStance.seek_range），不同士兵可能不同。cell_size 使用固定值 64（覆盖 192 单位），对 seek_range > 192 的情况做 fallback 全量扫描。

### Decision 4: arrow_movement_system 特殊处理

箭矢系统有穿透机制（hit_units 记录已命中）、城市碰撞、from_faction 阵营判断。SpatialHash 查询返回候选后，需额外过滤已命中实体和同阵营实体。

## Risks / Trade-offs

**[Risk] cell_size 选择不当导致漏查** → query_nearby 返回 9 cell，对 seek_range > 3*cell_size 的情况 fallback 全量扫描

**[Risk] arrow_movement_system 改动复杂度高** → 箭矢有穿透、城市碰撞等特殊逻辑，需仔细保持原有语义

**[Risk] 确定性** → BTreeMap + cell 内 UnitId 排序 + faction_map 查找，数学上完全确定（3 轮复核确认）
