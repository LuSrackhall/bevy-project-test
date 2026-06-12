## MODIFIED Requirements

### Requirement: HUD 布局

render_view crate SHALL 提供统一选中面板：左 30% 区同时承载城池和兵种信息（根据选中类型切换），右 70% 命令卡片 + 兵种图鉴。SHALL NOT 为城池和兵种使用不同面板位置。

#### Scenario: 选中城池时显示城池信息

- **WHEN** 玩家点击己方城池
- **THEN** 左面板切换为城池信息（等级/HP/人口/EXP/兵种按钮），兵种选中清空

#### Scenario: 选中兵种时显示兵种信息

- **WHEN** 玩家选中兵种
- **THEN** 左面板切换为兵种信息（单选全属性/多选聚合），城池选中清空

### Requirement: 选中互斥

城池和兵种 SHALL NOT 同时被选中。点击城池 SHALL 清空所有兵种选中。框选 SHALL 仅选中 `SoldierMarker` 实体，忽略 `CityMarker` 实体。

#### Scenario: 框选不选中城池

- **WHEN** 玩家框选区域包含城池建筑和兵种
- **THEN** 只有兵种被选中，城池被忽略

#### Scenario: 点击城池清空兵种

- **WHEN** 玩家框选了 3 个兵种后点击城池
- **THEN** 兵种选中全部清空，仅城池被选中
