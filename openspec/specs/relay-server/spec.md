# relay-server Specification

## Purpose
TBD - created by archiving change network-command-stream. Update Purpose after archive.
## Requirements
### Requirement: Relay Server collects player inputs per tick

The relay SHALL accept `PlayerTickFrame` messages from all connected clients. It SHALL buffer all received commands grouped by `(tick, player_id)`.

#### Scenario: collect input from one player

- **WHEN** relay receives `PlayerTickFrame { tick: 100, player_id: 1, commands: [cmd_a], .. }`
- **THEN** it SHALL store `cmd_a` under `buffer[100][1]`

#### Scenario: collect input from multiple players

- **WHEN** relay receives frames for tick 100 from players 1, 2, and 3
- **THEN** it SHALL store each player's commands independently

### Requirement: Relay finalizes tick when all players ready or timeout

The relay SHALL finalize a tick when either all active expected player inputs have been collected, or the tick's timeout has elapsed. A player marked `Disconnected` SHALL be treated as satisfied for the all-ready check — the tick SHALL finalize via the timeout path for that player (with NoOp injection) rather than hanging the barrier indefinitely.

#### Scenario: finalize when all players arrive

- **WHEN** players 1, 2, 3 have submitted frames for tick 100
- **THEN** relay SHALL finalize tick 100 immediately (without waiting for timeout)

#### Scenario: finalize on timeout with NoOp injection

- **WHEN** player 2 has not submitted frame for tick 100 within the timeout window
- **THEN** relay SHALL finalize tick 100 with `NoOp` injected for player 2

#### Scenario: disconnected player does not hang the barrier

- **WHEN** player 2 has disconnected and is marked `Disconnected`, and players 1, 3 have submitted frames for tick 100
- **THEN** relay SHALL finalize tick 100 (player 2 satisfied via timeout, NoOp injected) and NOT wait forever

## ADDED Requirements


### Requirement: Relay broadcasts finalized CommandBatch

Once a tick is finalized, the relay SHALL broadcast a `CommandBatch` (wrapped in `BroadcastFrame`) to all connected clients.

- The batch SHALL contain all commands sorted by `(player_id, sort_tag)`
- The batch SHALL contain commands for ALL players (including each player's own commands — echo)
- The batch SHALL be immutable once broadcast (no late corrections)

#### Scenario: broadcast includes all players

- **WHEN** tick 100 is finalized with commands from players 1, 2 and NoOp for player 3
- **THEN** the broadcast `CommandBatch` SHALL contain commands for players 1, 2, and 3

#### Scenario: late input deferred to next tick

- **WHEN** tick 100 has already been finalized and broadcast
- **THEN** any late `PlayerTickFrame` for tick 100 SHALL be stored as input for tick 101 (or rejected)

### Requirement: Relay deduplicates by (tick, player_id, player_sid)

The relay SHALL reject duplicate `PlayerTickFrame` messages with the same `(tick, player_id, player_sid)` combination to prevent duplicate command injection.

#### Scenario: duplicate frame rejected

- **WHEN** relay receives two `PlayerTickFrame` messages with identical `(tick=100, player_id=1, player_sid=42)`
- **THEN** the second SHALL be silently dropped

### Requirement: Relay caches command log for reconnect

The relay SHALL store all finalized `CommandBatch` entries for the duration of the game session. This log SHALL be retrievable by reconnecting clients.

#### Scenario: reconnect fetches log

- **WHEN** a client sends `ReconnectRequest { game_id, last_tick_consumed: 50 }`
- **THEN** relay responds with `ReconnectResponse` containing `ticks: Vec<TickCommands>` from tick 51 to current

### Requirement: Relay does NOT simulate or modify commands

The relay SHALL NOT run simulation, assign ordering keys, modify command payloads, or inspect `Action` variants for branching decisions.

#### Scenario: relay does not inspect action payload

- **WHEN** a `PlayerTickFrame` contains commands with various `Action` variants
- **THEN** the relay SHALL forward them without inspection, modification, or conditional behavior based on action type

### Requirement: Relay retains seats for disconnected players

The relay SHALL retain a player's seat after disconnect (mark `Disconnected`), not permanently remove them from the expected player set. On reconnect, the player SHALL be re-admitted under their original `player_id`.

#### Scenario: disconnected player keeps seat

- **WHEN** player 2 disconnects mid-game
- **THEN** the relay SHALL keep player 2's seat (expected-player set unchanged), mark them `Disconnected`, and continue finalizing ticks with NoOp for player 2

#### Scenario: lobby drop does not deadlock the room

- **WHEN** player 2 drops in the lobby (before signaling ready) and is marked `Disconnected`
- **THEN** the all-ready check SHALL exclude player 2, so the remaining players can still start the game

#### Scenario: reconnect reuses original player id

- **WHEN** the disconnected player 2 reconnects
- **THEN** the relay SHALL assign the same `player_id` 2 (not `Room is full`), remove the `Disconnected` mark, and resume accepting their frames


### Requirement: Lobby ready tracking is scalable

The relay's lobby ready tracking SHALL NOT be limited to 8 players. Ready state SHALL be tracked per-player without a bit-mask ceiling.

#### Scenario: 9 players all ready

- **WHEN** 9 players each signal `LobbyReady`
- **THEN** the relay SHALL track all 9 as ready and start the game when all are ready (no bit-mask overflow)



### Requirement: Relay manages UDP sessions by source address

relay SHALL 维护每客户端 UDP 会话表:单 socket 共享 recv 后按源地址去复用报文,路由到对应会话的可靠状态。JoinGame 建立完整会话,端口变化时重发 JoinGame 更新源地址。

#### Scenario: session tracked by source address

- **WHEN** 客户端从某源地址发 JoinGame 建立会话
- **THEN** relay SHALL 将会话记录在会话表(源地址 → 会话),后续该地址报文路由到对应会话

#### Scenario: port change updates session address

- **WHEN** 客户端因 NAT rebinding 从新端口重发 JoinGame
- **THEN** relay SHALL 更新会话的源地址,继续识别同一玩家

### Requirement: Heartbeat timeout triggers disconnect

relay SHALL 通过心跳超时判定客户端掉线(替代 TCP 连接断开检测):会话的 `last_seen` 超阈值(如心跳间隔 × N)后,标记 `Disconnected`(席位保留,靠 NoOp 推进)并触发 `on_disconnect`。超时阈值 SHALL 大于正常抖动,避免误判活客户端。

#### Scenario: heartbeat timeout marks disconnected

- **WHEN** 客户端心跳超时(超过阈值未收到)
- **THEN** relay SHALL 判定该会话掉线,标记 `Disconnected`(席位保留)并触发 `on_disconnect`

#### Scenario: jitter does not falsely kill a live client

- **WHEN** 客户端因网络抖动心跳延迟(但在阈值内)
- **THEN** relay SHALL NOT 误判掉线,会话保持活跃

### Requirement: Relay re-sends GameStarted to reconnecting players in a started game

When a client joins a game that has already started AND its JoinGame reuses a `Disconnected` seat (i.e., it is a reconnecting player whose process restarted), the relay SHALL respond with `GameJoined`, then send `GameStarted` (with seed) again, before the reconnect pages. Clients already Playing SHALL ignore the duplicate `GameStarted` (only the lobby transition handles it).

#### Scenario: restarted process gets GameStarted on reconnect

- **WHEN** a player's process restarts, reconnects (JoinGame reuses the Disconnected seat) to a game that is already started
- **THEN** the relay SHALL send `GameStarted { seed, player_count }` to that player after `GameJoined`, so its lobby can transition to Playing and rebuild with the reconnect metadata seed

#### Scenario: duplicate GameStarted is harmless to live clients

- **WHEN** the relay re-sends `GameStarted` to a reconnecting player while other clients are Playing
- **THEN** the Playing clients SHALL ignore the duplicate (no state reset, no world rebuild)

---
**Implementation:** `network.rs` lines 238-514 (RelayServer state machine). `relay_core.rs` + `ReliableSocket`(UDP 会话表/心跳清扫). 6 unit tests covering state machine logic. 3 UDP integration tests in `relay/tests/integration.rs`. 1 Bevy e2e test in `bevy_adapter/tests/network_e2e.rs`.

