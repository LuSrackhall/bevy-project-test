# github-actions-ci Specification

## Purpose
TBD - created by archiving change setup-github-actions. Update Purpose after archive.
## Requirements
### Requirement: CI workflow SHALL trigger on push and PR to main

CI 检查流水线 MUST 在以下条件下触发：push 到 main 分支，或 PR 目标为 main 分支。

#### Scenario: Push to main triggers CI
- **WHEN** 代码推送到 main 分支
- **THEN** CI workflow 在 Linux、macOS、Windows 三平台上并行执行

#### Scenario: PR to main triggers CI
- **WHEN** 创建或更新目标为 main 的 Pull Request
- **THEN** CI workflow 在 Linux、macOS、Windows 三平台上并行执行

### Requirement: CI SHALL perform format check on Linux only

`cargo fmt --check` MUST 仅在 Linux runner 上执行一次。

#### Scenario: Format check runs once
- **WHEN** CI workflow 执行
- **THEN** 仅在 ubuntu-latest 上运行 `cargo fmt --check`

### Requirement: CI SHALL run clippy on all three platforms

`cargo clippy -- -D warnings` MUST 在 Linux、macOS、Windows 上各执行一次。

#### Scenario: Clippy runs on all platforms
- **WHEN** CI workflow 执行
- **THEN** 在 ubuntu-latest、macos-latest、windows-latest 上分别运行 `cargo clippy -- -D warnings`

### Requirement: CI SHALL run tests with GPU-safe configuration

MUST 分两步执行测试：`cargo test -p simulation`（无 GPU 依赖）单独运行，`cargo test --workspace` 设置 `WGPU_BACKEND=noop` 环境变量。

#### Scenario: Simulation tests run safely
- **WHEN** CI workflow 执行
- **THEN** 在三平台上运行 `cargo test -p simulation` 且不依赖 GPU

#### Scenario: Workspace tests run in headless mode
- **WHEN** CI workflow 执行
- **THEN** 在三平台上运行 `cargo test --workspace` 且设置 `WGPU_BACKEND=noop`

### Requirement: CI SHALL build workspace on all platforms

`cargo build --workspace` MUST 在三平台上执行。

#### Scenario: Workspace builds succeed
- **WHEN** CI workflow 执行
- **THEN** 在三平台上成功运行 `cargo build --workspace`

### Requirement: CI SHALL install Linux system dependencies

Linux runner MUST 安装 Bevy 所需的系统依赖：`libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev libvulkan-dev pkg-config`。

#### Scenario: Linux dependencies installed
- **WHEN** CI workflow 在 ubuntu-latest 上执行
- **THEN** 通过 apt 安装所有 Bevy 系统依赖

### Requirement: CI SHALL use Rust cache

MUST 使用 `Swatinem/rust-cache@v2` 进行 Cargo 缓存，并设置 `CARGO_INCREMENTAL=0`。

#### Scenario: Cache is utilized
- **WHEN** CI workflow 执行
- **THEN** Cargo 依赖缓存被正确使用

