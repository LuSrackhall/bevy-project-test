## ADDED Requirements

### Requirement: RelayRuntime trait

SessionController 通过 RelayRuntime trait 创建 relay 实例，实现与具体启动方式的解耦。

#### Scenario: ThreadRelayRuntime successfully starts relay

- **WHEN** `ThreadRelayRuntime::start()` 被调用，传入有效的 `RoomMetadata`
- **THEN** 返回 `Ok(Box<dyn RelayHandle>)`，其中 `endpoint()` 返回一个有效端口（127.0.0.1:<随机端口>）

#### Scenario: ThreadRelayRuntime fails gracefully

- **WHEN** `ThreadRelayRuntime::start()` 因端口冲突或其他系统错误失败
- **THEN** 返回 `Err(RelayError::StartFailed)`，不 panic

### Requirement: RelayHandle trait

RelayHandle 提供对运行中 relay 实例的控制，包括查询身份、获取连接地址、优雅关闭。

#### Scenario: RelayHandle returns relay_id

- **WHEN** 调用 `handle.relay_id()` 
- **THEN** 返回创建时分配的 `RelayId`

#### Scenario: RelayHandle returns endpoint

- **WHEN** 调用 `handle.endpoint()`
- **THEN** 返回包含实际绑定端口的 `SocketAddr`

#### Scenario: RelayHandle shutdown stops relay

- **WHEN** 调用 `handle.shutdown()`
- **THEN** relay 线程正常退出，不再接受新连接；返回 `Ok(())`

### Requirement: Session struct

Session 组合 RoomMetadata 和 RelayHandle，表示一个正在运行中的房间。

#### Scenario: Session construction

- **WHEN** 用 `RoomMetadata` 和 `Box<dyn RelayHandle>` 构造 `Session`
- **THEN** 通过 `session.room` 和 `session.relay` 可访问对应字段

### Requirement: SessionController lifecycle

SessionController 管理当前 Session 的创建与销毁，遵守 I1（单 Session）。

#### Scenario: Create session when none active

- **WHEN** `SessionController` 没有活跃 Session，调用 `create_session(room)`
- **THEN** relay 启动成功；返回 `Ok(&Session)`；`is_active()` 返回 true

#### Scenario: Create session when one already active

- **WHEN** `SessionController` 已有活跃 Session，再次调用 `create_session(room)`
- **THEN** 先关闭当前 Session，再创建新的；最终只有一个活跃 Session

#### Scenario: Destroy session

- **WHEN** `SessionController` 有活跃 Session，调用 `destroy_session()`
- **THEN** relay 停止；`is_active()` 返回 false；`current_session()` 返回 None

#### Scenario: Destroy when no session

- **WHEN** `SessionController` 没有活跃 Session，调用 `destroy_session()`
- **THEN** 返回 `Ok(())`（空操作，不报错）

### Requirement: RoomMetadata 创建时确定

SessionController 创建 Session 时传入的 RoomMetadata 在 Session 生命周期内不修改。

#### Scenario: RoomMetadata fields are frozen after creation

- **WHEN** `create_session(room)` 成功后
- **THEN** `session.room.room_id`、`session.room.map_id`、`session.room.max_players` 不可变

### Requirement: RelayError types

统一的 error 枚举，覆盖 relay 生命周期中的各类失败场景。

#### Scenario: StartFailed error

- **WHEN** relay 启动过程中发生错误
- **THEN** 返回 `RelayError::StartFailed(String)`，消息描述具体原因

#### Scenario: ShutdownFailed error

- **WHEN** relay 关闭过程中发生错误
- **THEN** 返回 `RelayError::ShutdownFailed(String)`，消息描述具体原因
