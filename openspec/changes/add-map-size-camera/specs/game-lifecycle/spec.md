## MODIFIED Requirements

### Requirement: NeedsGameReset 标志
`NeedsGameReset` SHALL 从 `bool` 改为枚举：`None`（暂停恢复）、`SameSize`（重开）、`NewGame(MapSize)`（新游戏）。

#### Scenario: 首次开始游戏
- **WHEN** 用户点击"大"地图按钮
- **THEN** `NeedsGameReset` 设为 `NewGame(MapSize::Large)`

#### Scenario: 暂停恢复
- **WHEN** 用户点击"继续"
- **THEN** `NeedsGameReset` 保持 `None`

#### Scenario: 重开游戏
- **WHEN** 用户点击"重新开始"
- **THEN** `NeedsGameReset` 设为 `SameSize`
