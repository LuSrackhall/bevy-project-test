## MODIFIED Requirements

### Requirement: Bottom panel shows selected city data
选中己方城池后，底部面板 SHALL 显示城池的实时数据（人口）。等级、血量、经验已迁移到单位头顶信息条显示，不再在底部面板重复。取消选中后面板隐藏。

#### Scenario: Select player city
- **WHEN** 玩家选中一座己方城池，人口 15/45
- **THEN** 底部面板显示：`[城池] 兵 15/45`（不含等级、血量、经验）

#### Scenario: Deselect city
- **WHEN** 玩家点击空地取消城池选中
- **THEN** 底部面板隐藏（display: None）

### Requirement: Soldier info in bottom panel
底部面板在选中士兵时 SHALL 显示攻击力、速度和特殊技能。血量、经验已迁移到单位头顶信息条显示，不再在底部面板重复。

#### Scenario: Select soldier
- **WHEN** 玩家选中一个士兵（攻击 15、速度 3.0、无特殊技能）
- **THEN** 底部面板显示攻击和速度信息（不含血量、等级、经验）

## REMOVED Requirements

### Requirement: HP bar rendered as Bevy Node
**Reason**: 血量显示已迁移到单位头顶世界空间信息条（血条 + 数值），不再在屏幕空间底部面板渲染血条 Node。
**Migration**: 无迁移需要——血量条已通过 unit-overhead-info-bar 在单位头顶以 ShapeBundle 矩形形式呈现。
