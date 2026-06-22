## ADDED Requirements

### Requirement: .cargo/config.toml SHALL configure WASM RUSTFLAGS

`.cargo/config.toml` MUST 为 wasm32-unknown-unknown target 配置 `getrandom_backend=wasm_js` 和 `-Ctarget-feature=+atomics`。

#### Scenario: WASM RUSTFLAGS are configured
- **WHEN** 对 wasm32-unknown-unknown target 执行构建
- **THEN** RUSTFLAGS 包含 `--cfg getrandom_backend="wasm_js"` 和 `-Ctarget-feature=+atomics`

### Requirement: Trunk.toml SHALL specify build target and tools

`Trunk.toml` MUST 指定 `target = "index.html"`、`dist = "dist"`，并配置 `wasm_opt` 工具版本。

#### Scenario: Trunk build uses correct configuration
- **WHEN** 运行 `trunk build`
- **THEN** 使用 index.html 作为入口，输出到 dist 目录，并使用 wasm_opt 优化

### Requirement: index.html SHALL serve as WASM entry point

`index.html` MUST 包含 Trunk 所需的 `data-trunk rel="rust"` 指令，作为 WASM 构建入口。

#### Scenario: Trunk recognizes index.html
- **WHEN** Trunk 读取 index.html
- **THEN** 正确识别 Rust WASM 构建指令并编译项目

### Requirement: .gitignore SHALL exclude dist and Trunk artifacts

`.gitignore` MUST 排除 `dist/` 目录。

#### Scenario: Build artifacts not tracked
- **WHEN** 运行 trunk build
- **THEN** dist/ 目录不被 git 跟踪
