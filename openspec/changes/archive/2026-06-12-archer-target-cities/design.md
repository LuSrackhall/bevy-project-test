## Context

`archer_attack_system` 当前目标搜索仅查询 `SoldierMarker` 实体。无结果时直接恢复 `SoldierState::Moving`，弓兵不发射。

已有 `arrow_movement_system` 城池碰撞逻辑（1/200 伤害累积），无需重复实现伤害。

## Goals / Non-Goals

**Goals:**
- 弓兵在无敌方兵种时，搜索射程内敌方城池作为备选目标
- 城池目标使用城池中心位置（`LogicalPosition`）

**Non-Goals:**
- 不改变箭矢对城池的伤害模型（保持 1/200）
- 不改变兵种优先的搜索顺序

## Decisions

### D1: 城池搜索放在兵种搜索的 fallback 位置

搜索优先级：
1. 射程内最近敌方兵种（现有逻辑）
2. 若无，射程内最近敌方城池（新增）
3. 若无，恢复 Moving（现有逻辑）

城池仅在无敌方兵种时被瞄准，弓兵不会因城池存在而忽略敌方兵种。

### D2: 城池搜索使用 CityMarker 过滤

收集 `CityMarker + LogicalPosition + FactionComponent` 实体，与兵种搜索模式一致。

### D3: 城池位置直接用中心点

城池无"边缘"坐标，使用 `LogicalPosition` 作为瞄准点。箭矢扩散机制仍适用。
