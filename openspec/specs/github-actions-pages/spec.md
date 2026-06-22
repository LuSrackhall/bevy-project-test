# github-actions-pages Specification

## Purpose
TBD - created by archiving change setup-github-actions. Update Purpose after archive.
## Requirements
### Requirement: Pages workflow SHALL trigger after Release completes

Pages 部署流水线 MUST 通过 `workflow_run` 在 Release workflow 完成后自动触发。

#### Scenario: Pages deploys after release
- **WHEN** Release workflow 成功完成
- **THEN** Pages workflow 自动触发并部署 WASM 版本

### Requirement: Pages SHALL deploy WASM build artifact

MUST 下载 Release workflow 产出的 WASM artifact 并通过 `actions/deploy-pages` 部署到 GitHub Pages。

#### Scenario: WASM artifact is deployed to Pages
- **WHEN** Pages workflow 执行
- **THEN** WASM 构建产物被部署到 GitHub Pages 站点

### Requirement: Pages site SHALL be accessible at user's GitHub Pages URL

站点 MUST 可通过 `https://luSrackhall.github.io/bevy-project-test/` 访问。

#### Scenario: Site is accessible
- **WHEN** 部署完成
- **THEN** 可通过 GitHub Pages URL 在浏览器中访问游戏

