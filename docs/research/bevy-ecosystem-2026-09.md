# Bevy 生态现状调研（检索基准日 2026-09-12）

> 方法说明：本会话 `web_search` 不可用（缺 API key），全部结论来自 `web_fetch` 直接抓取的
> bevy.org 官方页面、GitHub REST API、raw.githubusercontent.com 原文。
> 下文【事实】= 抓取到的原文/数据；【推断】= 我的判断。

## 0. 结论摘要（TL;DR）

1. **版本不是问题**：0.19.1（2026-08-13）就是最新稳定版，锁定 0.19 已经等于最新。
   真正的痛点是"每 3.5–5 个月一次的大型 API 迁移"（0.18→0.19 迁移指南 93KB），以及 **0.20 正在开发中**
   （`main` = `0.20.0-dev`）。Bevy **无 LTS、无稳定化承诺**。
2. **编译慢可以量化治理**：官方基准显示渲染占 76% 冷编译时间，其中 `bevy_pbr` 单项 43.6s。
   本项目用 `bevy` 默认 feature（含 3D），把它改成 `default-features=false, features=["2d","ui"]`
   预计省掉约 35–40% 冷编译时间——**这是投入产出比最高的一步**。
   0.19 之后没有任何编译性能改进 PR。
3. **UI：bevy_ui 仍不足以独立支撑复杂 HUD**，Feathers 官方自述非游戏用途且实验性。
   现实选择是 bevy_egui（最快成型）或 bevy_ui + bevy_flair（CSS/热重载）。
4. **网络：lightyear 0.29 是当前最成熟的确定性锁步方案**（唯一支持 input-only 确定性复制 +
   lockstep/rollback 双模式 + 已上 0.19）；bevy_replicon 只能做状态同步补充；naia 已落后一个版本。
5. **Agent 测试：Bevy 明确不做测试框架**（issue #2896 以 `not_planned` 关闭，docs 无 testing.md，
   `/learn/book/testing/` 404）。可行组合是三层：① `bevy_ecs`-only 的 `simulation` +
   确定性回放 hash 断言（主战场）；② 官方 `bevy_remote`(BRP) 的 JSON-RPC/OpenRPC 做运行时状态级断言
   与 agent 自发现；③ 若要 UI 像素级 E2E，唯一官方路径是 `wasm-pack test --headless`，
   Playwright 需自建（把 HUD 放浏览器即可白拿 Playwright + 官方 MCP server）。
6. **不建议换引擎**。换 fyrox/godot-rust 不解决编译慢（godot-rust 还多绑一个 C++ 引擎）。
   若一定要动，本项目 `render_view` 已有 wasm-bindgen/web-sys 依赖，
   **"Rust 仿真核 + 浏览器 Web HUD" 是六条路线里边际成本最低、且同时命中痛点 1/3/4 的一条**。


## 1. 版本现状

【事实】
- 最新稳定版 **0.19.1**，发布于 **2026-08-13**（GitHub Releases API）。
- **0.19.0** 发布于 **2026-06-18/19**；0.18.0 → 2026-01-13；0.18.1 → 2026-03-02；
  0.17.0 → 2025-09-30；0.16.0 → 2025-04-24。
- 0.19 之后**没有新的 minor 版本**。`main` 分支 `Cargo.toml` 为 `version = "0.20.0-dev"`。
- 进行中的里程碑：**0.19.2**（4 open / 12 closed）、**0.20**（8 open / 121 closed，2026-09-11 更新）、
  **0.21**（15 open / 1 closed，2026-08-31 新建）。
- 发布节奏：官方公告原文称 "Since our last release **a few months** ago"。
  实测间隔 0.16→0.17 约 159 天、0.17→0.18 约 105 天、0.18→0.19 约 156 天。
- **无 LTS / 无稳定化承诺**：官方 Setup 文档写明 MSRV 是 "the latest stable release" of Rust；
  GitHub 上检索 "LTS"/"long-term support" 无任何官方政策条目（仅命中 Ubuntu LTS 噪音）。
- 迁移成本可量化的代理指标（官方迁移指南文件体积，来自 bevy-website 仓库）：
  0.15→0.16 = 1.0 KB；**0.16→0.17 = 129.7 KB（最大）**；0.17→0.18 = 55.6 KB；
  **0.18→0.19 = 93.3 KB**。
- 0.19 核心变化：新场景系统 **BSN**（`bsn!`/`SceneComponent`，暂无官方 `.bsn` 资源加载器）、
  Feathers 组件扩充（text input / list view / scrollbar / dropdown）、
  `EditableText` 文本输入、字体族与可变字重、App Settings、渲染大幅提速
  （many_cubes 1.6M 立方体 49.47ms → 18.77ms）、`bevy_city` 19.3ms → 11.8ms。
- 0.18 核心变化：Cargo feature 集合（`2d`/`3d`/`ui`）、标准 widget 首批扩展
  （Popover/Menu）、Feathers ColorPlane、`bevy_camera_controller`、字体变体、
  glTF Extension handler、`EasyScreenshot`/`EasyScreenRecord`。

【推断】
- 该项目 `Cargo.toml` 锁 **0.19 实际上已经是最新版**（仅差一个 patch）。"跟不上新版本" 的真实成本
  不是落后，而是**每 3.5~5 个月一次、体积巨大的 API 迁移**（0.18→0.19 指南 93KB，
  0.16→0.17 达 130KB）。所以正确的对策是**降低迁移成本**而非追赶版本。
- 0.20 尚未发布（无 RC tag），按 cadence 估计落在 2026-10~2027-01 区间。**这是推断，非官方日期。**

## 2. 编译速度

【事实】
- 官方 Setup 文档："**Dynamic Linking** — This is the **most impactful** compilation time decrease!"，
  用法 `--features bevy/dynamic_linking`。限制被官方明确列出：
  ① Windows 上必须同时开性能优化否则报 "too many exported symbols"；
  ② `cargo test --doc` 需把 `rustc --print target-libdir` 加入 `PATH`；
  ③ **发布时不要开**（需附带 `libbevy_dylib`、阻碍优化、增大体积）。
- 官方推荐 profile：`[profile.dev] opt-level = 1` + `[profile.dev.package."*"] opt-level = 3`；
  release 用 `codegen-units = 1`、`lto = "thin"`。
- 链接器：**mold 比 LLD 快最多 5×**；LLD 已默认用于 Windows，Linux 上 LLD 是 rustc 默认；
  **macOS 上系统默认 ld-prime 比 LLD 更快，应继续用默认**。
- Cranelift（nightly）：比 LLVM **约快 30%**，Linux 最佳，**macOS 上 Bevy 可能崩溃**。
- `-Zcache-proc-macros`（nightly）：**增量编译时间降低 5–10%**（config_fast_builds.toml 注释）。
- `[profile.dev] debug = 1`：官方注明 "In most cases the gains are negligible, but if you are on
  macOS and have slow compile times you should see significant gains"。
  注：官方 fast-build 配置里**没有 `split-debuginfo`**。
- **量化冷编译数据**（issue #23642，2026-04-03 提交，**至今 open**，Linux / Ryzen 9 9950X3D）：
  `cargo clean && cargo build --timings --example 3d_scene`：
  `bevy_pbr` **43.61s**、`bevy_render` 25.15s、`bevy_core_pipeline` 17.04s、`bevy_gizmos_render` 5.4s
  → 渲染相关合计 **91.2s = 总编译时间的 76%**，其中一半是 `bevy_pbr`。
  与 Bevy ~0.12 对比：**当前总编译时间是 0.12 的约 6 倍**，`bevy_pbr` 单项是 0.12 的 **17 倍**。
- 项目自身的编译优化 PR：**#20647**（merged 2025-12-16，进入 0.18）"Optimize
  `BundleInserter::insert` compile time"，clean compile **92s → 82s（-10%）**。
- **0.19 之后（2026 年）没有检索到编译性能改进 PR**：以
  `repo:bevyengine/bevy compile in:title is:pr merged:>2026-01-01` 检索，9 条结果全部是编译**错误**
  修复或 rust-version 调整，无编译**速度**改进。0.19 里程碑（2026-06-17 closed）同。
- 0.18 起提供 **Cargo feature 集合**：`bevy = { version = "…", default-features = false,
  features = ["2d"] }`；`2d` → `2d_bevy_render` 只含 `bevy_render`/`bevy_core_pipeline`/
  `bevy_post_process`/`bevy_sprite_render`/`bevy_gizmos_render`，**不含 `bevy_pbr`、`bevy_gltf`**。
- Cargo.toml 中 `default_app` 注释原文：可作为 "headless apps that require no rendering
  (ex: command line tools, servers, etc)" 的基线。
- 0.19.1 共 **171 个 feature flag，默认开 74 个**；其中包含 `bevy_ci_testing`
  （"Enable systems that allow for automated testing on CI"）、`bevy_feathers`、
  `bevy_ui_widgets`、`bevy_remote`、`dynamic_linking`。
  来源：https://docs.rs/crate/bevy/0.19.1/features

【推断】
- 对该 RTS 最高性价比的组合（按证据强度排序）：
  ① **改用 `default-features = false, features = ["2d", "ui"]`** —— 直接砍掉 `bevy_pbr`，
     按 #23642 的数据可省掉总冷编译的 ~35–40%；
  ② `dynamic_linking`（迭代编译收益最大，但只用于本地 dev，CI/发布必须关）；
  ③ macOS 上别折腾 mold/lld，改用 `debug = 1`；Linux CI 用 mold + `-Zcache-proc-macros`。
- 结构上更彻底的做法：把 `simulation` 做成**不依赖 bevy 渲染栈**的 crate（仅 `bevy_ecs` 或
  `bevy` + `default_app`），让仿真/测试的编译与渲染解耦——这同时解决痛点 1 和痛点 4。
- 未查到 `dynamic_linking` 增量的**官方量化数字**（官方只说 "most impactful"），
  也未查到社区权威的统一基准。**此项暂无可靠数据，不编造。**

### 2b. 本仓库现状核对（本地文件事实）
- 根 `Cargo.toml`: `bevy = "0.19"` —— **使用默认 feature**，即同时编译 2d + **3d（`bevy_pbr`/`bevy_gltf`）**
  + ui + audio。`crates/{bevy_adapter,presentation,render_view}` 三个 crate 同样 `bevy = "0.19"` 默认 feature。
- `crates/simulation`: 只依赖 `bevy_ecs 0.19` + `default-features = false`（编译开销极小，且天然 headless）。
- `crates/render_view`: 已有 `cfg(target_arch = "wasm32")` 依赖 `wasm-bindgen`/`js-sys`/`web-sys`
  （`HtmlInputElement`、`CssStyleDeclaration`、`KeyboardEvent`）。

【推断】② 对该项目最直接的编译优化：把 `bevy_adapter`/`presentation`/`render_view` 的 bevy
改为 `default-features = false, features = ["2d", "ui"]`（按 #23642 数据，可去掉约 35–40% 冷编译时间，
且 2D RTS 并不需要 `bevy_pbr`）。此举风险低、可先用 `cargo build --timings` 验证前后差异。

## 3. UI 生态

（详见同目录 `bevy-ui-ecosystem-2026-09.md`）

【事实】
- bevy_ui 0.19 的实际进步：`EditableText`（光标/选区/系统剪贴板/IME/多行软换行+垂直滚动/输入过滤/字数上限，
  但**无 placeholder、无 undo-redo、无密码掩码**）；`FontSource`（Handle / 字体族名 / 语义类别）+
  可变字重 + OpenType features；`FlexWrap`（PR #25449，2026-08-18）；核心 Scrollbar +
  Feathers list view/scrollbar；交互状态组件、theme 框架、focus ring、圆角。
- **查不到**上游的列表虚拟化能力；0.19 **未发布** `.bsn` 资源加载器 → **没有资源驱动的 UI 热重载**。
- Bevy Feathers：0.19 新增 text/number input、dropdown menu、disclosure toggle、list view、scrollbar、pane。
  但官方文档明确它面向未来的 Bevy Editor，**"deliberately not intended for" 游戏 UI**，
  且 "still experimental and unfinished! It will change in breaking ways"。
  跟踪 issue #19236 显示 tooltip/toast/tab/tree view/notification 等仍未完成；
  issue #25500（2026-08-21 开 → 2026-09-02 关）报告 `feathers_gallery` debug 模式卡顿、slider 掉帧。
- 备选库（版本 / 日期 / Stars / Bevy / 状态）：
  | 库 | 版本·日期 | Stars | Bevy | 状态 |
  |---|---|---|---|---|
  | bevy_egui | 0.42.0 · 2026-08-16（egui 0.36） | 1413 | 0.19 | 活跃，0.40.0 与 Bevy 0.19 同期发布 |
  | egui 本体 | 0.36.2 · 2026-09-08 | 30518 | 引擎无关 | 活跃 |
  | bevy_lunex | 0.7.0 · 2026-08-31 | 959 | 0.19 | 活跃（自研 retained 布局） |
  | bevy_flair | 0.8.1 · 2026-08-21 | 155 | 0.19 | 活跃（CSS + 热重载 + @media/@layer/transition） |
  | belly | 0.4.0 · 最后推送 2024-04-20 | 433 | 0.13 | **停滞** |
  | kayak_ui | 0.5.0 · 2024-02-11 | 484 | 0.12 | **弃用** |
  | sickle_ui | 0.4.0 · 2024-10-03 | 仓库 404 | 0.14 | README 声明**不再公开维护**（被 0.15 淘汰） |
  | bevy_cobweb_ui / rxy_ui | — | 83 / 65 | — | **已 archived** |
- 混合架构：`bevy_cef` 0.12.0（2026-07-08，51★，支持 0.19）把 CEF 离屏渲染到 3D mesh / 2D sprite 并支持
  JS↔Bevy 双向与 DevTools，代价是 CEF 体积与打包；`blaind/bevy_webview`（最后提交 2023-12-10）、
  `bevy_webview_projects`、`bevy_dioxus` 均已死。`jf908/State-of-Bevy-Webviews` 把
  "Bevy→wasm + HTML overlay" 列为形态之一。
  **"WASM 游戏核心 + 浏览器 HTML/CSS 前端 IPC" 的公开项目：查不到。**
- **社区没有官方/共识推荐文档**（查不到）。

【推断】③ 对"信息密集 HUD + 大量交互面板"：
- 最快成型 → **bevy_egui**（星最多、跟版最快、自带 side_panel/split_screen/multi-window 面板范式）；
  代价是渲染风格与游戏美术不统一、多一个渲染通道。
- 保留模式 + CSS 主题/热重载 → **bevy_ui + bevy_flair**，但 list/tab/tooltip/虚拟滚动需自建。
- **Feathers 不要当游戏 HUD 依赖**（官方自述非游戏定位 + 实验性 + 已知 debug 性能问题），只作实现参考。
- 本项目 `render_view` 已存在 `wasm-bindgen`/`web-sys`(HtmlInputElement/CssStyleDeclaration) 依赖，
  说明 **HTML overlay + 浏览器 UI 的边际成本最低**，且能直接复用 Playwright/MCP 做 agent 自动化
  （见第 5、6 节）。这是本项目相对社区通用建议的特殊优势。

## 4. 网络生态

【事实】（GitHub API 抓取，2026-09-12）
| 库 | 版本 / 发布 | 最近推送 | Stars | Bevy | 确定性锁步 / 回滚 |
|---|---|---|---|---|---|
| lightyear | 0.29.0 / 2026-08-10 | 2026-09-12 | 1145 | 0.19 | ✅ 确定性 input-only 复制，兼容 **lockstep 与 prediction/rollback**；0.29 新增确定性 P2P（要求 peer 数固定）；0.27+ 支持 late-join 快照 catch-up |
| bevy_replicon | 0.44.0 / 2026-09-01 | 2026-09-11 | 648 | 0.19 | ❌ 仅服务端权威 state replication，README 无 lockstep/rollback |
| aeronet | 0.21.0 / 2026-06-24 | 2026-09-05 | 135 | 0.19 | ❌ 仅传输层；README 明确 replication/rollback 是 non-goals |
| naia | 0.25.0 / 2026-05-12 | main 分支 2026-05-13 后无提交 | 1176 | **0.18** | ❌ 服务端权威 + 预测 + 延迟补偿 |
| ggrs | 0.13.0 / 2026-06-26 | 2026-08-25 | 683 | 引擎无关 | ✅ P2P rollback(GGPO) **+ lockstep 模式**，含 desync 检测 |
| bevy_ggrs | 0.22.0 / 2026-06-26 | 2026-07-26 | 364 | 0.19 | ✅（GGRS 封装，已有 `box_game_synctest` 示例） |

lightyear README 原文："Deterministic replication … compatible with both **lockstep** and
prediction/rollback."

【推断】
- 2 人以上实时 RTS + 确定性锁步 + 命令驱动，**当前最成熟方案是 lightyear 0.29**
  （唯一同时具备确定性输入复制、lockstep/rollback 双模式、P2P/CS 拓扑、late-join catch-up 且已上 0.19 的库）；
  次选 `bevy_ggrs`（GGRS 的 lockstep 模式更"专为帧同步/desync 检测而生"，但需自建命令流与观战）。
- `bevy_replicon` 只能当状态同步补充层，**不能当锁步主通道**；`aeronet` 是 IO 层；
  `naia` 落后一个 Bevy 版本且 main 停滞，不建议新项目采用。
- 未检索到 2025–2026 新出现的、Bevy 专用确定性锁步 RTS 框架（"bevy netcode" 结果基本为停更/实验仓库）。

## 5. AI / agent 原生测试

【事实】
- **官方 headless 路径是文档化的**：`bevy::MinimalPlugins`（crate 根导出，
  https://docs.rs/bevy/0.19.1/bevy/struct.MinimalPlugins.html ）；
  官方 crate 文档把 `default_app` collection 描述为
  "useful as a baseline feature set for scenarios like **headless apps that require no rendering**
  (ex: command line tools, servers, etc)"（https://docs.rs/bevy/0.19.1/bevy/index.html ）。
  → 无渲染依赖意味着编译与运行都不需要 GPU/窗口，是 agent 可跑的纯逻辑测试基线。
- **0.19.1 内置 `bevy_ci_testing` feature**，描述原文："Enable systems that allow for automated
  testing on CI"（https://docs.rs/crate/bevy/0.19.1/features ）。这是官方 CI 自动化测试机制。
- **`bevy_remote` feature = Bevy Remote Protocol（BRP）— 本节最重要的发现**。官方原文：
  "These _remote clients_ can **inspect and alter the state of the entity-component system**"，
  "The Bevy Remote Protocol is based on the **JSON-RPC 2.0** protocol"
  （https://docs.rs/bevy/0.19.1/bevy/remote/index.html ）。0.19.1 内置方法：
  `world.query`（支持 `components`/`option:"all"`/`has` + `with`/`without` 过滤）、
  `world.get_components`、`world.spawn_entity`、`world.despawn_entity`、
  `world.insert_components`、`world.remove_components`、`world.mutate_components`（按字段 path）、
  `world.reparent_entities`、`world.list_components`、`world.get_components+watch`、
  `world.list_components+watch`、`world.get_resources`/`insert_/remove_/mutate_/list_resources`、
  `world.trigger_event`、`registry.schema`、**`rpc.discover`（返回 OpenRPC 文档，即机器可读的 API 自描述）**。
  传输无关（PR #23367，merged 2026-04-04，进入 0.19），HTTP 只是可选传输；可用
  `RemotePlugin::with_method` 注册自定义方法。前提：组件/资源需 `Reflect` 注册。
- **官方 headless 的做法（文档化但不成框架）**：`MinimalPlugins` = TaskPoolPlugin + FrameCountPlugin
  + TimePlugin + **ScheduleRunnerPlugin**（+ 启用 `bevy_ci_testing` 时加 `CiTestingPlugin`）；
  `ScheduleRunnerPlugin::{run_once(), run_loop(Duration)}` **不在 DefaultPlugins 中**；
  官方示例 `examples/app/headless.rs` 用 `DefaultPlugins.set(ScheduleRunnerPlugin::run_once())`。
  官方 CI 插件 `bevy_dev_tools::ci_testing::CiTestingPlugin` 读 `ci_testing_config.ron`
  （或 `CI_TESTING_CONFIG` 环境变量）脚本化事件（截图/退出），并可用
  `TimeUpdateStrategy::ManualDuration` 固定帧步长。
- **官方没有 test harness，且已被明确放弃**：issue **#2896 "Provide a way to test Bevy App"
  以 `not_planned` 关闭（2025-02-05）**，配套 PR #7314 关闭未合并；
  仓库 `docs/` 下**没有 testing.md**（只有 cargo_features/debugging/linters/linux_dependencies/profiling）；
  `https://bevy.org/learn/book/testing/` → **404**；crates.io 上的 `bevy_test` 只是**占位名**
  （v0.0.1，2020-08-14，零功能）。
- 实际可用的官方级原语：`App::update()`（公开方法，Bevy 自身单测即 `app.update()` 循环）与
  `World::run_system_once`（`bevy_ecs`）。**但没有任何官方文档把它定为推荐测试模式**——属事实性空白。
- **`bevy_debug_stepping` feature**："Enable stepping-based debugging of Bevy systems"
  → 可按系统逐步执行，天然适合可复现的确定性测试。
- 确定性回放：**GGRS `SyncTestSession` 是纯本地、无网络的确定性自测**——每帧回滚并重跑最近
  `check_distance` 帧后比对 checksum（`SessionBuilder::with_check_distance(7).start_synctest_session()`）；
  `bevy_ggrs` 暴露 `Session::SyncTest` + `SyncTestMismatch` observer + `checksum_component_with_hash::<T>()`，
  并附 `docs/debugging-desyncs.md`（列非确定查询顺序、`Local<T>`、f32 跨平台、RNG、`Res<Time>` 等成因）。
- 浏览器自动化：**官方支持路径是 `wasm-bindgen-test` + `wasm-pack test --headless --chrome/--firefox/--safari`**
  （WebDriver；文档已迁至 https://wasm-bindgen.github.io/wasm-bindgen/wasm-bindgen-test/browsers.html ，
  旧 rustwasm.github.io 于 2025-07-21 停止维护）。**Playwright/Puppeteer 测 Bevy 的公开博客：查不到。**
- **最接近"agent 自主验证"的公开先例**：`linonetwo/bevy-visual-e2e-testing-ci-example`
  （Bevy 0.17.2，pushed 2026-03-01）——cucumber + reqwest，**在游戏进程内嵌 MCP JSON-RPC over
  HTTP(:9222)**，暴露 `take_snapshot`/`click_by_id`/`component_counts`/`screenshot`，
  目标是让 Copilot 自己跑 e2e 验证改动。注意：它走 **MCP/HTTP 而非 wasm-bindgen 导出**。
- Bevy MCP 生态很小：`mcp-bevy-buddy` 8★、`Grok-Bevy`（BRP query/screenshot，2026-07，0★）、
  `bevy_mcp`/`bevy-mcp` 0★、`bevy_brp_mcp` 已归档。对比 Godot MCP 生态大得多
  （`Coding-solo/godot-mcp` 5643★；另有 455★ 的"157 tools"实现）。
- **明确查不到的项（not found，不作推测性填补）**：
  ① 官方 Bevy 测试 harness / 官方测试指南；
  ② Bevy + Playwright/Puppeteer 的公开方案或博客；
  ③ 用 wasm-bindgen 把游戏状态导出给 JS/agent 做断言的公开样例；
  ④ "WASM 游戏核心 + 浏览器 HTML/CSS 前端 IPC" 的公开 Bevy 项目；
  ⑤ 完全无人在环的"agent 自主开发并验证游戏逻辑"公开项目或论文（现有先例都需人在环）。
- 2026 进展：**Bevy Editor 尚未发布**；0.19 的 What's Next 提到 "Entity inspector … framework has been
  prototyped"、`.bsn` 资产读写、以及 "a much more complete Bevy book … during the 0.20 development cycle"。
  均属路线图，**不可作为测试方案依赖**。

【推断】⑤ 对"测试由 AI agent 原生自动化（无人类介入）"的可落地组合：
- **逻辑层（最高优先级，且完全不需要 App）**：`simulation` 只依赖 `bevy_ecs`，
  直接 `World::new()` + 手动 tick 驱动即可被 agent 完整驱动；再加"同种子 + 同命令序列 → 全量状态 hash
  比对"就是最可靠的 agent 自验证闭环（语义等价于 GGRS SyncTest，无需引入 GGRS）。
  这条路不依赖任何 Bevy 渲染/UI，成本最低、确定性最高。
- **BRP 是最贴合"agent 原生"的官方通道**：`bevy_remote` 让 agent 通过 JSON-RPC 直接 query/mutate
  运行中的 ECS，`+watch` 订阅变更、`registry.schema` 取类型结构，而
  **`rpc.discover` 返回 OpenRPC 文档 = 机器可读的 API 自描述**，agent 可零人工介入地发现可用方法与参数。
  这比截图/像素比对稳定一个数量级，且绕开 bevy_ui 成熟度问题。代价：官方无面向 agent 的封装，
  组件需 `Reflect` 注册。→ 建议把"BRP 客户端 + MCP server 薄封装"作为痛点 4 的主攻方向
  （与上述 linonetwo 先例的思路一致，但用官方 BRP 替代自制协议）。
- **UI/E2E 层**：Bevy 侧没有等价于 Playwright 的闭环；若必须做像素级 E2E，
  `wasm-bindgen-test`/`wasm-pack test --headless` 是唯一有官方支持的路径，
  而 Playwright + 官方 MCP 需要自建（本项目 `render_view` 已有 wasm-bindgen/web-sys 依赖，成本最低）。

## 6. 横向对比

（详见同目录 `engine-alternatives-2026-09.md`）

【事实】各方案基线数据：
- **macroquad** 0.4.16（2026-07-30）·4624★·2026 仍在维护；README 自述 `cargo clean` 后构建 **16s**（老机器）；
  自带 immediate-mode UI，但第三方 `egui-macroquad` 停在 0.17.3（2025-05-21，128★，绑 egui 0.31）。
- **Fyrox** v1.0.0（2026-03-29，**首个稳定版**）·9548★·pushed 2026-09-12；自带编辑器 + 保留式 UI，
  提供 `export-cli` 可接 CI/CD，release notes 提到 editor/automated tests。**构建耗时：查不到。**
- **godot-rust (gdext)** v0.5.5（2026-08-09）·5182★；官网称 binary compatibility 低至 Godot 4.1；
  0.5.3（2026-05-19）移除 bindgen/LLVM/C 编译器需求，godot-core 编译约快 7%；
  测试生态是 **GUT 9.7.1**（CLI + JUnit XML + VSCode 扩展），测试脚本用 GDScript。
- **Bevy 基线** v0.19.1·48146★·3428 open issues；官方文档直言 "the compile times are rather long"。
- **Rust 核 + WASM/Web**：`wasm-pack` 7284★；rustwasm book 有官方 polyglot 教程；真实案例 Rerun。
  **RTS 级 "Rust 核 + TS UI" 成品项目：查不到。**
- **纯 TypeScript**：Playwright v1.63.0（2026-09-04）·96001★，官网自述面向 **"AI agents"**；
  **官方 MCP server `microsoft/playwright-mcp` 37028★**（pushed 2026-09-11）+ agent CLI。
  Node 26 的 type stripping 已 stable（可直接 `node file.ts` 零构建）；
  esbuild 官方基准 three.js×10 打包 **0.39s vs webpack5 41.21s**。

三轴评分（**全部是我的推断，1–5 分**）：

| 方案 | (a) 编译速度 | (b) HUD UI | (c) agent 可测性 |
|---|---|---|---|
| macroquad | 5 | 2 | 2 |
| Fyrox | 3（无数据） | 4 | 3 |
| godot-rust | 3 | 5 | 4 |
| **Bevy（现状）** | **2** | **3** | **3** |
| Rust 核 + WASM + Web | 3 | 5 | **5** |
| 纯 TypeScript | 5 | 5 | **5** |

【推断】④
- 三轴综合最优是 **纯 TS** 或 **Rust 核 + Web 前端**；Playwright 官方 MCP/CLI 是唯一**明确为 coding agent
  设计且仍在高频维护**的 E2E 闭环，Bevy 侧无等价物。
- 换 TS 能同时解决四个痛点，但代价是**丢掉 Rust 的类型安全与定点数保证**：JS 的 `number` 是 f64，
  与当前 `Fixed(i64)` 确定性架构冲突——纯 TS 路线必须引入整数/定点库，lockstep 才成立。这是本路线
  最大的技术风险，**不是性能问题**。
- 若必须保留 Rust 逻辑核（本项目 `simulation` 已是纯 `bevy_ecs` + `Fixed(i64)`，宪法天然隔离渲染），
  最优是 **WASM + Web 前端**：仿真核不动，只把 `render_view` 换成浏览器 DOM/CSS HUD + Playwright。
  本项目 `render_view` **已经**有 `wasm-bindgen`/`web-sys` 依赖，是六条路线里**边际成本最低**的一条，
  且同时命中痛点 1（仿真核编译与渲染解耦）、3（DOM/CSS 做 HUD 远强于 bevy_ui）、4（Playwright+MCP）。
- 不值得为 UI 全面换引擎（fyrox/godot-rust）：换引擎不解决"编译慢"（godot-rust 还多绑一个 C++ 引擎），
  只是把 UI 问题换成另一套生态锁定。

