## 1. 模块骨架

- [x] 1.1 创建 `bevy_adapter/src/session_host/` 目录，包含 mod.rs、controller.rs、runtime.rs、error.rs、thread.rs
- [x] 1.2 在 `bevy_adapter/src/lib.rs` 中注册 `pub mod session_host`
- [x] 1.3 在 `bevy_adapter/Cargo.toml` 中确认 tokio 依赖（验证 `rt`、`net`、`io-util`、`time`、`macros`、`sync`）

## 2. RelayError

- [x] 2.1 定义 `pub enum RelayError` 包含 `StartFailed(String)` 和 `ShutdownFailed(String)` 变体，实现 `std::fmt::Display` 和 `std::error::Error`

## 3. RelayRuntime trait + RelayHandle trait

- [x] 3.1 定义 `pub trait RelayRuntime`：`fn start(&mut self, room: &RoomMetadata) -> Result<Box<dyn RelayHandle>, RelayError>`
- [x] 3.2 定义 `pub trait RelayHandle`：`fn relay_id(&self) -> RelayId`、`fn endpoint(&self) -> SocketAddr`、`fn shutdown(self: Box<Self>) -> Result<(), RelayError>`

## 4. Session + SessionController

- [x] 4.1 定义 `pub struct Session { pub room: RoomMetadata, pub relay: Box<dyn RelayHandle> }`
- [x] 4.2 定义 `pub struct SessionController { runtime: Box<dyn RelayRuntime>, session: Option<Session> }`
- [x] 4.3 实现 `SessionController::new(runtime)`、`is_active()`、`create_session(room)`、`current_session()`、`destroy_session()`

## 5. ThreadRelayRuntime + ThreadRelayHandle

- [x] 5.1 实现 `ThreadRelayRuntime`：bind 127.0.0.1:0 获取 OS 分配端口，spawn 线程 + tokio runtime，调用 relay 逻辑，通过 channel 返回实际端口
- [x] 5.2 实现 `ThreadRelayHandle`：存储 `RelayId`、`SocketAddr`、停止信号 `AtomicBool`、线程 `JoinHandle`
- [x] 5.3 实现线程安全退出：`shutdown()` 设置停止信号 + 通知 relay 停止 + 等待线程退出

## 6. 单元测试

- [x] 6.1 测试 `SessionController::create_session` 正常路径
- [x] 6.2 测试 `SessionController::destroy_session`（有 session / 无 session）
- [x] 6.3 测试 `create_session` 在已有 session 时自动替换
- [x] 6.4 测试 `RelayError` Display 格式
- [x] 6.5 测试 `Session` 字段可访问

---

## Post-Implementation Workflow

<!-- DO NOT MODIFY THIS SECTION — it defines the required workflow after all tasks are complete -->

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/<name>`

**Iteration**: If user does not accept, analyze the issue and recommend:
fix in place / new change / git reset + stash / git reset / abandon.
