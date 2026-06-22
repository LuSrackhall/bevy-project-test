## Why

项目当前没有任何 CI/CD 配置，每次构建和发布都需要手动操作。利用 GitHub Actions 免费资源可以实现自动化的跨平台 CI 检查、二进制产物发布和 WASM 在线试玩部署，显著提升开发效率和测试便利性。

## What Changes

- 新增 CI workflow（`ci.yml`）：push/PR 到 main 时在 Linux/macOS/Windows 三平台并行执行 fmt、clippy、test、build 检查
- 新增 Release workflow（`release.yml`）：tag `v*` 触发，构建 7 个 target（macOS ARM/Intel、Windows x64/ARM、Linux x64/ARM、WASM）的二进制产物，上传到 GitHub Release
- 新增 GitHub Pages workflow（`pages.yml`）：Release 完成后自动部署 WASM 版本到 GitHub Pages
- 新增构建配置文件：`.cargo/config.toml`（WASM RUSTFLAGS）、`Trunk.toml`（WASM 构建）、`index.html`（WASM 入口）、`Cross.toml` + `docker/Dockerfile.cross-aarch64`（Linux ARM 交叉编译）
- 新增 `robots.txt` 阻止搜索引擎索引仓库

## Capabilities

### New Capabilities
- `github-actions-ci`: GitHub Actions CI 检查流水线（三平台并行 fmt/clippy/test/build）
- `github-actions-release`: GitHub Actions Release 构建与发布流水线（7 target 矩阵构建 + GitHub Release 创建）
- `github-actions-pages`: GitHub Pages WASM 自动部署（Release 后触发，部署 WASM 构建产物）
- `wasm-build-config`: WASM 构建基础设施（Trunk.toml、index.html、.cargo/config.toml RUSTFLAGS 配置）
- `cross-compile-linux-arm`: Linux ARM 交叉编译配置（Cross.toml + 自定义 Dockerfile）

### Modified Capabilities
（无现有 capability 的需求变更）

## Impact

- **新增文件**：`.github/workflows/ci.yml`、`.github/workflows/release.yml`、`.github/workflows/pages.yml`、`.cargo/config.toml`、`Trunk.toml`、`index.html`、`Cross.toml`、`docker/Dockerfile.cross-aarch64`、`robots.txt`
- **仓库设置**：需在 GitHub Settings > Pages 中将 Source 设为 "GitHub Actions"；需手动清除 Description/Topics 降低可发现性
- **已知风险**：Windows ARM target 的 wgpu 存在已知编译问题，可能需要从矩阵中移除；WASM 构建需要浏览器支持 COOP/COEP 头才能使用 SharedArrayBuffer
