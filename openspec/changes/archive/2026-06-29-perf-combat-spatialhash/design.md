## Context

4 个战斗系统 O(n²) 全量扫描。上次 SpatialHash 推广失败（借用冲突），本次用预构建 faction_map 解决。

## Goals / Non-Goals

**Goals:** 4 个系统全部改用 SpatialHash，O(n²) → O(n*k)

**Non-Goals:** 不改游戏逻辑、不改宪法

## Decisions

### 每系统独立 SpatialHash（不同 cell_size）

| 系统 | 范围 | cell_size |
|------|------|-----------|
| combat_engagement | seek_range (~150) | 64 |
| melee_attack | 30 | 32 |
| archer_attack | 380-600 | 200 |
| arrow_movement | 22 | 32 |

### 预构建 faction_map

从 all_units 拆出 HashMap<UnitId, Faction>，SpatialHash 循环内只读 map 不碰 World。

### combat_engagement seek_range fallback

cell_size=64 覆盖 192 单位。若 seek_range > 192，fallback 全量扫描。

### arrow_movement 特殊处理

穿透过滤（hit_units）、城市碰撞、from_faction 阵营判断。

## Risks / Trade-offs

**[Risk] cell_size 选择不当** → fallback 全量扫描兜底

**[Risk] arrow_movement 复杂度高** → 保持原有语义，只替换内层循环
