## ADDED Requirements

### Requirement: 命令不重复发送

`network_flush_system` 必须确保每个命令只发送一次，不跨帧重复。

#### Scenario: 命令取出即销毁
- **WHEN** `network_flush_system` 读取 `cmd_buf` 中 tick=N 的命令
- **THEN** 这些命令必须从 `cmd_buf` 中移除，确保下帧不再含入 PlayerTickFrame

#### Scenario: 空帧不覆盖有效数据
- **WHEN** `take_for_tick` drain 后下一帧发送空帧
- **THEN** relay 侧必须追加（extend）而非覆盖，空帧不干扰已有数据
