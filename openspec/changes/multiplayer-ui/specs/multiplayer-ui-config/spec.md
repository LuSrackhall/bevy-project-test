## ADDED Requirements

### Requirement: 玩家数量选择器
玩家数量按钮响应点击事件，在 2、3、4 之间循环。

#### Scenario: 点击玩家数量按钮（初始 2）
- **WHEN** 玩家点击 NetworkPlayerCount 按钮，当前值为 2
- **THEN** 按钮 Text 显示 "3"，NetworkPlayerCount 变为 3
- **AND** NetworkPlayerId clamp：若当前 ID ≥ 3，ID 自动回退为 1，对应 Text 同步更新

#### Scenario: 再次点击玩家数量按钮（当前 3）
- **WHEN** 玩家点击 NetworkPlayerCount 按钮，当前值为 3
- **THEN** 按钮 Text 显示 "4"，NetworkPlayerCount 变为 4

#### Scenario: 点击回起点（当前 4）
- **WHEN** 玩家点击 NetworkPlayerCount 按钮，当前值为 4
- **THEN** 按钮 Text 显示 "2"，NetworkPlayerCount 变为 2
- **AND** NetworkPlayerId clamp：若当前 ID ≥ 2，ID 自动回退为 1，对应 Text 同步更新

### Requirement: 玩家序号选择器
玩家 ID 按钮响应点击事件，在 0 到 (count-1) 之间循环。

#### Scenario: 点击玩家 ID 按钮（玩家数 4）
- **WHEN** 玩家点击 NetworkPlayerId 按钮，当前值为 0，count=4
- **THEN** 按钮 Text 显示 "1"，NetworkPlayerId 变为 1

#### Scenario: 点击到最大值后回 0
- **WHEN** 玩家多次点击 NetworkPlayerId 按钮至值为 3，count=4
- **THEN** 再次点击后按钮 Text 显示 "0"，NetworkPlayerId 变为 0

### Requirement: 开始按钮正确读取配置
开始联机按钮的 Query 能正确跨兄弟实体读取所有网络配置组件。

#### Scenario: 设置后开始联机
- **WHEN** 玩家设置 count=4、id=2，点击"开始联机"
- **THEN** `NeedsGameReset::Network` 的 player_count=4、player_id=2
