## ADDED Requirements

### Requirement: 命令指向未来 tick

网络模式下，新创建的命令必须指向 `current_tick + input_delay`，而非 `current_tick + 1`。

#### Scenario: 右键移动
- **WHEN** 玩家在 network 模式右键点击发出 MoveTo
- **THEN** 命令的 tick 必须为 `current_tick + input_delay`
- **AND** 该命令必须被 relay 接收并包含在 finalize 的广播中

### Requirement: 窗口帧保持 relay 推进

`network_flush_system` 必须为 `[current_tick+1, current_tick+input_delay]` 的每个 tick 发送一个帧。

#### Scenario: 中间 tick
- **WHEN** `input_delay = 3` 且 `current_tick = N`
- **THEN** 必须发送 tick N+1, N+2, N+3 的帧
- **AND** relay 能顺序 finalize 这些 tick

### Requirement: 输入先于网络发送

`command_issue_system` 必须在 `network_flush_system` 之前运行。

#### Scenario: 同帧点击
- **WHEN** 玩家点击且两系统同帧运行
- **THEN** 命令在同帧被 network_flush_system 发送

### Requirement: 命令可靠送达

网络模式下，命令必须可靠送达 simulation，不能因 tick 已 finalize 而丢弃。

#### Scenario: e2e 可靠性测试
- **WHEN** 双客户端逐 tick 发出 MoveTo 命令
- **THEN** 所有已处理 tick 的命令必须出现在 simulation 的注入命令日志中
