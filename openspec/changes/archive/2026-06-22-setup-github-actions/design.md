## Context

项目 `city-conquest`（Bevy 0.19 RTS 游戏）当前无 CI/CD 配置。仓库为公开 GitHub 仓库，可免费使用 GitHub Actions。本次变更需要建立完整的 CI/CD 流水线，包括三平台 CI 检查、多 target Release 构建、WASM GitHub Pages 部署。详细设计见 brainstorm-spec.md。

## Goals / Non-Goals

**Goals:**
- 三平台 CI 检查自动化
- 多平台 Release 二进制产物自动构建与发布
- WASM 版本自动部署到 GitHub Pages
- 最小化仓库可发现性

**Non-Goals:**
- 移动端完整构建
- 自动版本号管理
- 代码覆盖率

## Decisions

### 1. CI 仅在 main 的 push/PR 触发

避免 feature 分支频繁触发 CI，节省 runner 时间。fmt 检查只在 Linux 上执行一次（格式与平台无关）。

### 2. Release 使用 7 target 矩阵构建

macOS ARM/Intel、Windows x64/ARM、Linux x64/ARM、WASM。Windows ARM 为实验性 target，wgpu 有已知编译问题。

### 3. Linux ARM 使用 cross + 自定义 Docker 镜像

cross 默认 Docker 镜像缺少 Bevy 系统依赖（libasound2-dev、libudev-dev、libwayland-dev、libxkbcommon-dev）。需要自定义 Cross.toml + Dockerfile。

### 4. WASM 使用 Trunk 构建

Trunk 是 Bevy WASM 构建的社区标准工具。需要 Trunk.toml、index.html、.cargo/config.toml 中的 RUSTFLAGS 配置（getrandom_backend + atomics）。

### 5. Pages 通过 workflow_run 触发

Release workflow 完成后自动触发 Pages 部署，解耦 Release 和 Pages 的逻辑。

### 6. CI 测试分两步

`cargo test -p simulation`（无 GPU 依赖）单独运行 + `cargo test --workspace` 设置 `WGPU_BACKEND=noop`。确保 CI 无 GPU 环境下测试不失败。

## Risks / Trade-offs

- **[Windows ARM wgpu]** → 如果编译失败，从矩阵中移除
- **[WASM SharedArrayBuffer]** → GitHub Pages 需要 COOP/COEP 头，首次部署时验证
- **[cross Docker 镜像]** → 首次构建较慢，后续有缓存
- **[Trunk workspace 兼容性]** → 可能需要显式指定 manifest path
