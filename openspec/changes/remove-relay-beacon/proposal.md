## Why

Change A 在 ThreadRelayRuntime 中实现了权威 UDP beacon。relay crate 残留的独立 beacon 在各方面均劣于权威版本（硬编码默认值、单目标广播、无停止信号、bind 失败 panic），需要删除。

## What Changes

- 删除 `crates/relay/src/lib.rs` 第 30-59 行 UDP beacon 代码
- 清理不再使用的 imports

## Capabilities

### New Capabilities
（无）

### Modified Capabilities
- `relay-server`: 删除残留 beacon 代码

## Impact

| 范围 | 文件 | 说明 |
|---|---|---|
| 修改 | `crates/relay/src/lib.rs` | 删除 beacon 代码 + 清理 imports |
| 无 | 构建脚本 / CI | 不引用 relay beacon |
| 无 | Cargo.toml | 不改依赖 |
