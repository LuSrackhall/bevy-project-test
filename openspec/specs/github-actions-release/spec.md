# github-actions-release Specification

## Purpose
TBD - created by archiving change setup-github-actions. Update Purpose after archive.
## Requirements
### Requirement: Release workflow SHALL trigger on tag push and manual dispatch

Release 构建流水线 MUST 在推送 `v*` 格式的 tag 或手动 `workflow_dispatch` 时触发。

#### Scenario: Tag push triggers release
- **WHEN** 推送名为 `v0.1.0` 的 tag
- **THEN** Release workflow 开始执行

#### Scenario: Manual dispatch triggers release
- **WHEN** 通过 GitHub UI 手动触发 workflow_dispatch
- **THEN** Release workflow 开始执行

### Requirement: Release SHALL build 7 platform targets

MUST 并行构建以下 target：macOS ARM (aarch64-apple-darwin)、macOS Intel (x86_64-apple-darwin)、Windows x64 (x86_64-pc-windows-msvc)、Windows ARM (aarch64-pc-windows-msvc)、Linux x64 (x86_64-unknown-linux-gnu)、Linux ARM (aarch64-unknown-linux-gnu)、WASM (wasm32-unknown-unknown)。

#### Scenario: All targets build in parallel
- **WHEN** Release workflow 执行
- **THEN** 7 个 target 在对应 runner 上并行构建

#### Scenario: Windows ARM failure does not block other targets
- **WHEN** Windows ARM target 构建失败
- **THEN** 其余 6 个 target 的构建和 Release 创建不受影响

### Requirement: Release SHALL create GitHub Release with all artifacts

所有构建产物 MUST 通过 `softprops/action-gh-release@v2` 上传到 GitHub Release。

#### Scenario: Release is created with all artifacts
- **WHEN** 所有 target 构建完成
- **THEN** 创建 GitHub Release 并上传所有产物

### Requirement: Release artifact naming convention

产物 MUST 遵循命名规范：Unix 为 `city-conquest-{target}.tar.gz`，Windows 为 `city-conquest-{target}.zip`。

#### Scenario: Artifact names follow convention
- **WHEN** 构建产物上传
- **THEN** 文件名符合 `city-conquest-{target}.{ext}` 格式

### Requirement: Release SHALL use upload-artifact per platform

每个平台 MUST 通过 `actions/upload-artifact` 上传构建产物，汇总 job 下载所有 artifact 后统一创建 Release。

#### Scenario: Artifacts flow through upload/download pattern
- **WHEN** 单个平台构建完成
- **THEN** 通过 upload-artifact 上传；汇总 job 下载所有 artifact 后创建 Release

