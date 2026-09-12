# Bevy 替代方案调研（确定性 Lockstep RTS）— 2026-09

## 一、FACTS FOUND（原始事实，含 URL）

| 项目 | 最新版本 / 日期 | pushed_at | Stars | 关键事实 |
|---|---|---|---|---|
| macroquad | crates 0.4.16 / 2026-07-30 | 2026-08-18 | 4624 | 仓库未 archived；0.4.15 (2026-05-20)、0.4.16 (2026-07-30) → **2026 年仍在维护**。README："cargo clean 后构建仅 16s（x230 老笔记本）"、"Immediate mode UI library included"。docs.rs 构建 26s（历史均值 1m5s）。https://api.github.com/repos/not-fl3/macroquad, https://docs.rs/crate/macroquad/latest, https://raw.githubusercontent.com/not-fl3/macroquad/master/README.md |
| macroquad UI | egui-macroquad 0.17.3 / 2025-05-21 | — | 128 | 官方无 UI 工具包，第三方只到 egui 0.31（egui 本体已 0.36.2）。https://docs.rs/crate/egui-macroquad/latest ; egui 30507★ / 0.36.2 (2026-09-08) https://docs.rs/crate/egui/latest |
| Fyrox | v1.0.0 / 2026-03-29 发布（首个稳定版） | 2026-09-12 | 9548 | 自带编辑器 + 保留式 UI（新增 input box、文本省略号裁剪等）；提供 `export-cli` 供 CI/CD；release notes 提到"editor tests"/"automated tests"。open issues 58。https://api.github.com/repos/FyroxEngine/Fyrox, https://github.com/FyroxEngine/Fyrox/releases/tag/v1.0.0, https://fyrox.rs/blog/post/fyrox-game-engine-1-0-0/ 。**构建耗时：not found（查不到）** |
| godot-rust / gdext | v0.5.5 / 2026-08-09 | 2026-09-07 | 5182 | Godot 4 绑定；官网："binary compatibility down to Godot 4.1"。v0.5.4 (2026-06-23) 标注 `api-4-7` level（即对 Godot 4.7 API）；v0.5.3 (2026-05-19)：**"No more bindgen/LLVM/C compiler"，godot-core 编译时间约快 7%**。https://api.github.com/repos/godot-rust/gdext, https://github.com/godot-rust/gdext/releases, https://godot-rust.github.io/ |
| Godot 版本 / 测试 | GUT 9.7.1 → Godot 4.7.x 分支；main 分支 → 4.6.x | GUT 2026-08-18 | GUT 2724 | GUT 有 **CLI**、可导出 **JUnit XML**、有 VSCode 扩展；测试用 GDScript 写。Godot 文档含命令行运行/`--headless` 章节。https://raw.githubusercontent.com/bitwes/Gut/main/README.md, https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html |
| Bevy（基线） | v0.19.1 / 2026-08-13（0.19.0 2026-06-18） | 2026-09-11 | 48146 | 官方 setup 文档直言 **"the compile times are rather long"**，给出 dynamic_linking（"most impactful"）、lld/mold、cranelift（约快 30%）、nightly generic sharing 等缓解手段；MSRV=最新 stable。bevy_ui 0.19.1 (2026-08-13)，依赖 taffy 0.10，文档覆盖 88.6%、108 项中仅 12 项带示例；docs.rs 构建均值 2m54s。https://api.github.com/repos/bevyengine/bevy, https://bevy.org/learn/quick-start/getting-started/setup/, https://docs.rs/crate/bevy_ui/latest |
| Rust 核心 + WASM + Web UI | wasm-pack 7284★，pushed 2026-08-12 | — | — | 官方文档化范式：Rust+WASM+JS/HTML/CSS polyglot 开发与"设计 Rust↔JS API"。真实案例：Rerun（Rust 核心，README 提供 "Run the Rerun Viewer in your browser"）。https://api.github.com/repos/rustwasm/wasm-pack, https://raw.githubusercontent.com/rustwasm/book/master/src/game-of-life/introduction.md, https://raw.githubusercontent.com/rerun-io/rerun/main/README.md 。**"Rust 逻辑核 + TypeScript UI" 的 RTS 级成品项目：not found（查不到）** |
| 纯 TypeScript | Playwright v1.63.0 / 2026-09-04 | 2026-09-11 | 96001 | 官网自述 "reliable web automation for testing, scripting, and **AI agents**"；**官方 MCP server** microsoft/playwright-mcp（37028★，2025-03-21 创建，pushed 2026-09-11）+ 面向 coding agent 的 CLI；MCP README 建议 coding agent 优先用 CLI+SKILLS。Node 26.8.2：**type stripping 已 stable（v25.2/v24.12）**，可直接 `node file.ts` 无构建步骤。esbuild 官方基准：three.js×10 打包 0.39s（webpack 5: 41.21s）。https://playwright.dev/, https://api.github.com/repos/microsoft/playwright, https://api.github.com/repos/microsoft/playwright-mcp, https://raw.githubusercontent.com/microsoft/playwright-mcp/main/README.md, https://nodejs.org/api/typescript.html, https://esbuild.github.io/ |

## 二、三轴评分（**评分＝我的推断，非事实**）

| 方案 | (a) 编译速度 | (b) 信息密集 HUD 的 UI 成熟度 | (c) AI 无人在环可测试性 |
|---|---|---|---|
| macroquad | **5**（极小依赖，16s/26s 数据支撑，但无增量热重载数据） | **2**（仅 immediate-mode 绘图；egui 绑定停滞在 2025-05、128★） | **2**（无内建自动化/无头测试范式，native GL 窗口循环） |
| Fyrox | **3**（大引擎、39 万 KB 仓库；**构建耗时无公开数据**） | **4**（自带保留式 UI + 编辑器控件；生态/文档第三方少） | **3**（export-cli 可 CI；编辑器有自动化测试痕迹；无 headless 测试框架证据） |
| godot-rust (gdext) | **3**（godot-core 已去 bindgen/LLVM，快 7%；仍捆绑 C++ Godot 二进制，迭代= cargo + 引擎启动） | **5**（Godot Control 体系多年成熟，HUD/表格/容器齐全） | **4**（GUT CLI + JUnit XML + VSCode 扩展；但测试须由 Godot 进程驱动、用 GDScript 写） |
| Bevy（基线） | **2**（官方承认编译慢，需 dynamic_linking/lld/cranelift 打补丁） | **3**（bevy_ui 为独立 crate、taffy 布局、随版本演进；示例稀疏：12/108） | **3**（ECS 纯 Rust 单测极易；UI/E2E 无内建方案，需自建 headless+截图对比） |
| Rust 逻辑核 + WASM + Web UI | **3**（双工具链：cargo + wasm-pack，WASM 增量不及 TS HMR） | **5**（DOM/CSS/React，信息密集 HUD 无上限） | **5**（wasm-bindgen 导出游戏状态 + Playwright/MCP 直驱浏览器） |
| 纯 TypeScript | **5**（esbuild 0.39s 级；Node 26 可直接跑 .ts，零构建） | **5**（Web 平台即最强 UI 平台） | **5**（Playwright 1.63 + 官方 MCP + agent CLI，最成熟的 agent E2E 闭环） |

## 三、结论（推断）

- **纯粹的"编译速度 × UI × agent 可测试性"最优点是纯 TypeScript 或"Rust 核 + Web 前端"**：Playwright 官方 MCP/CLI 是唯一为 coding agent 明确设计并维护（2026-09 仍高频提交）的 E2E 闭环，而 Bevy 侧不存在等价物。
- **Bevy 的代价已官方承认**：编译慢是文档化事实，只能靠 dynamic_linking/mold/cranelift 缓解；而 bevy_ui 虽每版迭代（taffy 布局、88.6% 文档覆盖），示例密度（12/108）与 HUD 组件生态仍显著弱于 Web。
- **macroquad 在 2026 年"还活着但很薄"**：版本仍在发（0.4.16, 2026-07-30），但 UI 依赖的 egui-macroquad 停在 2025-05（128★），post-1.0 的 UI 诉求基本无解 → 只适合"逻辑核准 + 自绘 HUD"。
- **Fyrox 1.0.0 是最接近"开箱即用编辑器 + UI"的纯 Rust 选项**，且已有 export-cli 可接 CI；但其构建耗时无公开数据、社区规模约为 Bevy 的 1/5，UI/测试生态仍偏薄。
- **若坚持 Rust 逻辑核（固定点确定性、无 GC 抖动），推荐 godot-rust 或 WASM+Web 两条路线**：前者白拿 Godot 成熟 UI 与 GUT（代价：GDScript 驱动测试 + 捆绑 C++ 引擎）；后者白拿 Playwright/MCP 与 DOM HUD（代价：双工具链 + wasm-bindgen 边界设计）。注意本仓宪法要求 `simulation` 零渲染依赖，两种方案都能满足，而 Godot/GDScript 侧测试需注意别把逻辑泄进 `.gd` 脚本。
- **确定性提醒（推断）**：纯 TS 路线的数值必须是整数/定点，不能依赖 JS `number`(f64) 的浮点行为，否则 lockstep 回放会漂移。
