## ADDED Requirements

### Requirement: validate_commands in run_tick

在 run_tick 的命令排序之后、consume_commands_system 之前增加验证阶段。

#### Scenario: Player commands own units

- **WHEN** 玩家 A 向自己阵营的单位发出 MoveTo 命令
- **THEN** 命令通过验证，进入 consume_commands_system

#### Scenario: Player commands enemy units

- **WHEN** 玩家 A 试图向玩家 B 阵营的单位发出 MoveTo 命令
- **THEN** 命令被过滤，不进入 consume_commands_system

#### Scenario: NoOp always passes

- **WHEN** 命令是 `Action::NoOp`
- **THEN** 不经过阵营检查，直接通过

#### Scenario: Single-player compatibility

- **WHEN** `PlayerSlots` Resource 不存在（单人模式）
- **THEN** 使用 `FactionId(cmd.player_id)` 兜底，不影响单人游戏

#### Scenario: AI commands

- **WHEN** AI 发出的命令（`player_id` 对应 AI slot）
- **THEN** 通过验证（AI 的 slot 配置的 FactionId 与命令操作的单位一致）

### Requirement: GameJoined 更新 NetworkCommandSource

GameJoined 事件被主线程读取后更新 NetworkCommandSource.player_id。

#### Scenario: GameJoined updates source

- **WHEN** 收到 `GameJoined { player_id: 1 }`
- **THEN** `SimulationDriver.source`（NetworkCommandSource）的 `player_id` 更新为 1
- **AND** `LocalPlayerIdentity` 更新为 1

#### Scenario: CLI path unaffected

- **WHEN** 使用 `--relay --player-id 1` CLI 参数
- **THEN** `NeedsGameReset::Network { player_id: Some(1) }` 路径不受影响
