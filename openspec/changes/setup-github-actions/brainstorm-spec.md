# GitHub Actions CI/CD Pipeline 设计

## Context

项目 `city-conquest` 是一个基于 Bevy 0.19 的 RTS 游戏，workspace 结构（crates: simulation, bevy_adapter, presentation, render_view），当前没有任何 CI/CD 配置。仓库托管在 GitHub (`LuSrackhall/bevy-project-test`)，公开状态。用户希望利用 GitHub Actions 免费资源进行 CI 检查、Release 发布、WASM GitHub Pages 部署，同时尽可能降低仓库可发现性。

## Goals / Non-Goals

**Goals:**
- CI 检查：push/PR 时在三平台（Linux/macOS/Windows）执行 clippy、test、build
- Release 发布：tag `v*` 触发，构建 6 个 target 的二进制产物，上传到 GitHub Release
- GitHub Pages：Release 完成后自动部署 WASM 版本到 Pages，可在浏览器中试玩
- 仓库隐藏：添加 robots.txt 阻止搜索引擎索引
- 新增必要的构建配置文件（Trunk.toml, .cargo/config.toml, Cross.toml 等）

**Non-Goals:**
- 移动端（iOS/Android）完整构建（当前仅能做到编译检查，无签名/打包能力，待未来需要时再添加）
- 自动版本号管理（手动打 tag）
- 代码覆盖率报告
- Docker 容器化

## Decisions

### 1. CI Workflow (`ci.yml`)

**触发条件**：push 到 main、PR 到 main

**平台矩阵**：ubuntu-latest、macos-latest、windows-latest 三平台并行

**检查内容**：
- `cargo fmt --check` — 仅在 Linux 上执行（格式检查与平台无关，跑一次即可）
- `cargo clippy -- -D warnings` — 三平台各跑
- `cargo test -p simulation` — 三平台各跑（simulation 无 GPU 依赖，安全运行）
- `cargo test --workspace` — 三平台各跑，设置 `WGPU_BACKEND=noop` 启用无 GPU 模式
- `cargo build --workspace` — 三平台各跑

**Linux 系统依赖**（通过 apt 安装）：
```
libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev libvulkan-dev pkg-config
```

**缓存**：`Swatinem/rust-cache@v2`，设置 `CARGO_INCREMENTAL=0`

### 2. Release Workflow (`release.yml`)

**触发条件**：推送 `v*` 格式的 tag（如 `v0.1.0`），或手动 `workflow_dispatch`

**构建矩阵（6 个 target）**：

| Target | Runner | Target Triple | 产物格式 | 备注 |
|---|---|---|---|---|
| macOS ARM | macos-latest | aarch64-apple-darwin | .tar.gz | |
| macOS Intel | macos-13 | x86_64-apple-darwin | .tar.gz | |
| Windows x64 | windows-latest | x86_64-pc-windows-msvc | .zip | |
| Windows ARM | windows-11-arm | aarch64-pc-windows-msvc | .zip | 实验性，wgpu 有已知编译问题 |
| Linux x64 | ubuntu-latest | x86_64-unknown-linux-gnu | .tar.gz | |
| Linux ARM | ubuntu-latest + cross | aarch64-unknown-linux-gnu | .tar.gz | 需自定义 Cross.toml + Dockerfile |
| WASM | ubuntu-latest + Trunk | wasm32-unknown-unknown | .tar.gz | 需 Trunk.toml + index.html |

**Windows ARM 风险说明**：`aarch64-pc-windows-msvc` 的 wgpu 存在已知编译问题（shader 编译工具链 DXC/Naga 问题、缺少预编译 ARM64 库）。如果该 target 构建失败，可随时从矩阵中移除。

**Release 流程**：
1. 各平台并行构建二进制产物
2. 每个平台通过 `actions/upload-artifact` 上传产物
3. 汇总 job 通过 `softprops/action-gh-release@v2` 创建 GitHub Release 并上传所有产物

**产物命名规范**：
- Unix: `city-conquest-{target}.tar.gz`
- Windows: `city-conquest-{target}.zip`

### 3. GitHub Pages Workflow (`pages.yml`)

**触发条件**：Release workflow 完成后触发（`workflow_run`）

**流程**：
1. 下载 Release workflow 产出的 WASM artifact
2. 通过 `actions/deploy-pages` 部署到 GitHub Pages

**站点地址**：`https://luSrackhall.github.io/bevy-project-test/`

**前提条件**：需要在仓库 Settings > Pages 中将 Source 设为 "GitHub Actions"

### 4. 仓库隐藏措施

- 在仓库根目录添加 `robots.txt`，内容：
  ```
  User-agent: *
  Disallow: /
  ```
- 需用户手动在 GitHub 网页设置中操作：
  - 清除仓库 Description
  - 清除所有 Topics
  - 移除 Website URL（如果有）
  - 替换 Social preview image 为纯色或空白图

**局限性**：知道 URL 或精确搜索代码内容仍能找到仓库，这是 GitHub 平台的限制。

### 5. 新增配置文件

#### `.cargo/config.toml`

为 WASM target 配置 RUSTFLAGS：

```toml
[target.wasm32-unknown-unknown]
rustflags = ['--cfg', 'getrandom_backend="wasm_js"', '-Ctarget-feature=+atomics']
```

**说明**：
- `getrandom_backend=wasm_js`：simulation crate 使用 rand，WASM 构建需要 JS 后端提供随机数
- `+atomics`：wasm_js 后端需要 SharedArrayBuffer 支持，浏览器需要 COOP/COEP HTTP 头

#### `Trunk.toml`

Trunk 构建配置（workspace 项目需要显式指定入口）：

```toml
[build]
target = "index.html"
dist = "dist"

[tools]
wasm_opt = { version = "0.116" }
```

#### `index.html`

WASM 构建入口文件：

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8"/>
    <link data-trunk rel="rust" data-wasm-opt="z"/>
</head>
<body></body>
</html>
```

#### `Cross.toml` + `docker/Dockerfile.cross-aarch64`

Linux ARM 交叉编译需要自定义 Docker 镜像，因为 cross 默认镜像缺少 Bevy 系统依赖。

**Cross.toml**：

```toml
[target.aarch64-unknown-linux-gnu]
dockerfile = "docker/Dockerfile.cross-aarch64"
```

**docker/Dockerfile.cross-aarch64**：

```dockerfile
FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main

RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install -y \
    libasound2-dev:arm64 \
    libudev-dev:arm64 \
    libwayland-dev:arm64 \
    libxkbcommon-dev:arm64 \
    pkg-config

ENV PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
```

### 6. Release Profile（可选优化）

为 WASM 构建添加优化 profile，减小产物体积：

```toml
[profile.wasm-release]
inherits = "release"
opt-level = "z"
lto = "fat"
codegen-units = 1
strip = true
```

**注意**：`lto = "fat"` + `codegen-units = 1` 会显著增加 Release 构建时间（可能 10-30 分钟）。可以后续按需启用。

## Risks / Trade-offs

- **[Windows ARM wgpu 编译问题]** → 已知风险，wgpu 对 aarch64-pc-windows-msvc 支持不完善。如果该 target 构建失败，从矩阵中移除即可，不影响其他 target。
- **[WASM atomics + SharedArrayBuffer]** → 需要 COOP/COEP HTTP 头才能在浏览器中运行。GitHub Pages 默认不设置这些头，可能需要通过 `<meta>` 标签或自定义 404.html hack 解决。首次部署时验证。
- **[cross Docker 镜像构建时间]** → 首次构建 Linux ARM 时需要拉取 Docker 镜像并安装依赖，CI 时间较长（可能 5-10 分钟）。后续有缓存后会更快。
- **[Trunk workspace 兼容性]** → Trunk 可能无法正确识别 workspace 中的二进制 crate。如果遇到问题，需在 Trunk.toml 中显式指定 manifest path。
- **[仓库隐藏不彻底]** → GitHub 不支持"未列出"模式，只能降低可发现性，无法完全隐藏。
- **[cargo test GPU 依赖]** → 使用 `WGPU_BACKEND=noop` 绕过 GPU 依赖。如果 Bevy 0.19 不支持该环境变量，需要使用 `bevy` 的 `headless` feature 替代方案。
