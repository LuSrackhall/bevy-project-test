## Why

多人游戏中移动指令时灵时不灵，需多次点击才能生效。根因是 `network_flush_system` 每帧重复发送同一 tick 的命令，relay 无法去重并累加重复指令，导致最终 tick 中出现重复命令。

## What Changes

- `network_flush_system` 用 `take_for_tick` 替代 `filter().cloned()`，命令取出即销毁

## Capabilities

### New Capabilities
- `command-dedup`: 消除网络命令重复发送

### Modified Capabilities
无。不修改现有 spec 需求。

## Impact

- `crates/bevy_adapter/src/transport.rs`：~3 行改动
