## 1. WASM 构建基础设施

- [x] 1.1 创建 `.cargo/config.toml`，配置 wasm32-unknown-unknown target 的 RUSTFLAGS（getrandom_backend=wasm_js + atomics）
- [x] 1.2 创建 `Trunk.toml`，配置构建目标为 index.html，dist 输出目录，wasm_opt 工具版本
- [x] 1.3 创建 `index.html` 作为 WASM 构建入口，包含 Trunk 所需的 `data-trunk rel="rust"` 指令
- [x] 1.4 更新 `.gitignore` 添加 `dist/` 排除

## 2. Linux ARM 交叉编译基础设施

- [x] 2.1 创建 `Cross.toml`，为 aarch64-unknown-linux-gnu 指定自定义 Dockerfile 路径
- [x] 2.2 创建 `docker/Dockerfile.cross-aarch64`，基于 cross 默认镜像安装 arm64 系统依赖

## 3. CI Workflow

- [ ] 3.1 创建 `.github/workflows/ci.yml`，配置 push/PR 到 main 的触发条件
- [ ] 3.2 配置三平台矩阵（ubuntu-latest、macos-latest、windows-latest）并行执行
- [ ] 3.3 添加 Linux 系统依赖安装步骤（libasound2-dev、libudev-dev、libwayland-dev、libxkbcommon-dev、libvulkan-dev、pkg-config）
- [ ] 3.4 添加 Rust 缓存（Swatinem/rust-cache@v2）和 CARGO_INCREMENTAL=0
- [ ] 3.5 添加 fmt 检查（仅 Linux）、clippy、test（simulation 单独 + workspace 带 WGPU_BACKEND=noop）、build 步骤

## 4. Release Workflow

- [ ] 4.1 创建 `.github/workflows/release.yml`，配置 tag v* push 和 workflow_dispatch 触发条件
- [ ] 4.2 配置 7 target 构建矩阵（macOS ARM/Intel、Windows x64/ARM、Linux x64/ARM、WASM）
- [ ] 4.3 实现各 target 的构建步骤（Linux ARM 使用 cross，WASM 使用 trunk build）
- [ ] 4.4 配置 upload-artifact 上传各平台产物
- [ ] 4.5 创建汇总 job，下载所有 artifact 并通过 softprops/action-gh-release@v2 创建 Release

## 5. GitHub Pages Workflow

- [ ] 5.1 创建 `.github/workflows/pages.yml`，配置 workflow_run 触发（监听 Release workflow 完成）
- [ ] 5.2 实现下载 WASM artifact 并通过 actions/deploy-pages 部署的逻辑
- [ ] 5.3 配置所需的 permissions（pages: write、id-token: write）

## 6. 仓库隐藏

- [ ] 6.1 创建 `robots.txt`，阻止所有搜索引擎索引

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/setup-github-actions`
