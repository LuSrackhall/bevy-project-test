# Bevy UI 生态调研（RTS 信息密集型 HUD）— 快照日 2026-09-12

数据全部来自本次实际抓取。标注规则：**【事实】** 附 URL 与日期；**【推断】** 为我的判断；无法证实写 **查不到**。

## 1. 上游 bevy_ui（0.19.x）

**【事实】版本节奏**
- Bevy 0.19.0 发布 2026-06-19（261 贡献者 / 1185 PR）；0.19.1 于 2026-08-13 发布。https://bevy.org/news/bevy-0-19/ ；https://docs.rs/crate/bevy/latest
- 0.20 已在开发（bevy-assets 中官方示例标注 0.20.0-dev，抓取于 2026-09-12）。

**【事实】HUD 能力**
- 布局：`Node` flexbox；`FlexWrap` 存在（PR #25449，2026-08-18，用 flex-wrap 解决 gallery 溢出）。滚动：核心 widget 有 Scrollbar，Feathers 有 list view + scrollbar。**虚拟化长列表：查不到**（未见上游方案）。
- 文本：0.19 新增 `FontSource`（Handle / Family / 语义类别如 Monospace、SystemUi）+ 字体族与可变字体属性；新增 `EditableText`（光标、选区、多击选词、剪贴板、IME、多行软换行+垂直滚动、水平滚动、输入过滤、字数上限）。缺 placeholder / undo-redo / 密码掩码。来源同上公告。
- 样式与主题：上游已落地交互状态组件、主题框架、focus ring、圆角（tracking #19236 勾选项）；CSS 式样式需第三方。
- 热重载：`.bsn` 资源加载器 **0.19 未发布**（公告明言 future release），故上游无资源驱动的 UI 热重载；CSS 热重载由 bevy_flair 提供。

**【事实】bevy_feathers（第一方 widget 集）**
- 树内 crate，文档位于 https://docs.rs/bevy/latest/bevy/feathers/index.html （0.19.1，2026-08-13）。模块：containers / controls / display / cursor / focus / theme / tokens / dark_theme / font_styles / palette / rounded_corners。插件 `FeathersPlugins`、`FeathersCorePlugin` 可脱离编辑器独立加载。
- 0.19 新增 widget：text input、number input、dropdown menu button、menu divider、disclosure toggle、icon 与 label、pane/subpane/group、list view、scrollbar；并已迁移到 BSN。
- 定位原文：面向未来 Bevy Editor；"While it may be tempting to use this crate for your game's UI, it's deliberately not intended for that"；"still experimental and unfinished! It will change in breaking ways"。→ 可公开使用，但**实验性、非游戏 UI 定位**。
- 缺口（tracking #19236，2026-06-26 更新）：未完成 Discrete Scrubber、Notification、Tooltips、Infotips；Swatch Grid、Tab Deck、Tree View、Toasts；布局组件 Page layout / Toolbar / Sidebar（post-BSN）。
- 性能：【事实】issue #25500（2026-08-21 开，2026-09-02 关）报告 feathers_gallery 在 debug 下卡顿、slider 跳帧。

## 2. 备选库

| 库 | 最新版本(日期) | pushed_at | Stars | Bevy 支持 | 状态 |
|---|---|---|---|---|---|
| bevy_egui (vladbat00) | 0.42.0 (2026-08-16, egui 0.36) | 2026-08-31 | 1413 | 0.19 ↔ 0.40–0.42 | **活跃**（0.40.0 与 Bevy 0.19 同日 2026-06-19） |
| egui (emilk) | 0.36.2 (2026-09-08) | 2026-09-11 | 30518 | 引擎无关 | **活跃** |
| bevy_lunex | 0.7.0 (2026-08-31) | 2026-08-31 | 959 | 0.19 | **活跃**（retained 布局引擎） |
| bevy_flair (eckz) | 0.8.1 (2026-08-21)，0.8.0 于 2026-06-22 | 未查 | 155 | 0.19 ↔ 0.8 | **活跃**（CSS + 热重载） |
| belly (jkb0o) | 0.4.0（workspace，未上 crates.io） | 2024-04-20 | 433 | 0.13 | **停滞/弃用** |
| kayak_ui (StarArawn) | 0.5.0 (2024-02-11) | 2024-07-08 | 484 | 0.12 | **弃用** |
| sickle_ui (UmbraLuminosa) | 0.4.0 (2024-10-03) | 仓库 404 | 查不到 | 0.14 | README 原文：被 Bevy 0.15 变更淘汰，"will not be publicly maintained" |
| iyes_perf_ui | 未查 | 2025-05-20 | 236 | 未查 | 低频维护（bevy_ui 调试 HUD） |
| woodpecker_ui (StarArawn) | 未查 | 2026-07-21 | 74 | 未查 | 活跃（ECS 响应式 UI） |
| bevy_extended_ui (exepta) | 未查 | 2026-07-27 | 73 | 未查 | 活跃 |
| bevy_cobweb_ui (UkoeHB) | 未查 | 2026-01-14 | 83 | 未查 | **已 archive** |

来源：各仓库 `https://api.github.com/repos/...`（2026-09-12）、docs.rs crate 页（2026-09-12）、bevy_egui releases。注意 `mvlabat/bevy_egui` 已重定向到 `vladbat00/bevy_egui`。sickle_ui 的 `umut-sahin/sickle_ui` 与 `UmbraLuminosa/sickle_ui` 均 404。

## 3. Bevy Assets 页

**【事实】** https://bevy.org/assets/ 可抓取，站点导航含 **UI** 分类（同页还含 Accessibility、Development tools 等）；数据源仓库 `bevyengine/bevy-assets` 下 `Assets/UI/` 目录确实存在（`Assets/` 列表抓取于 2026-09-12）。**UI 分类的具体条目清单：查不到** —— 后续 `Assets/UI` 列表请求被 GitHub API 403（rate limit）拒绝，`github.com/.../tree/main/Assets/UI` HTML 抓取网络失败。

## 4. 推荐与 WASM/HTML 混合

**【事实】可见信号**
- Feathers 官方文档明确不面向游戏 UI；上游标准 widget 路线图（#19236）在 tooltip/toast/tab/tree/布局组件上仍未完成。
- bevy_egui 是星数最高且维护最快的集成（Bevy 发布当日跟进），README 直接提供 side_panel / split_screen / two_windows / render_egui_to_image 示例，天然契合"多面板 + 密集信息"。
- bevy_flair 提供 CSS 选择器、`@media`、`@layer`、transition/keyframes、变量与 **CSS 热重载**，但只解决"样式"，不提供 widget。
- 未见任何官方或社区"共识文档"；**查不到**明确推荐语。**【推断】** 事实上形成三分：bevy_ui(+flair) 保留模式但 widget 自建；bevy_egui 快速成型但美术风格/额外渲染通道受限；bevy_lunex 自定义布局换性能。

**【事实】HTML/CSS 混合**
- `jf908/State-of-Bevy-Webviews` README 明确把"编译 Bevy 到 wasm 后叠加 HTML UI"列为嵌入 webview 的替代方案，并指出会遇到浏览器 API 的兼容/性能/安全限制；同时称 webview 适合"用 Bevy 做性能敏感部分、用 web 技术做其余部分"。
- 反向方案（在原生 Bevy 窗口内渲染 HTML/CSS）：bevy_cef 0.12.0（2026-07-08，51★）支持 Bevy 0.19，CEF 离屏渲染到 3D mesh / 2D sprite，JS↔Bevy 双向通信、本地资源热重载、DevTools，多进程，macOS/Windows/Linux；代价是 CEF 框架打包与体积。blaind/bevy_webview（58★）最后提交 2023-12-10，已死；not-elm/bevy_webview_projects 已 archive；bevy_dioxus 仅 0.1.1（2022-07-14，Bevy 0.7），已死。
- **公开的"WASM 游戏核心 + 浏览器 HTML/CSS 前端 IPC 分离"项目：查不到**；公开存在的都是"把 web 技术嵌进 Bevy 窗口"（bevy_cef）或"wasm + DOM overlay"。
