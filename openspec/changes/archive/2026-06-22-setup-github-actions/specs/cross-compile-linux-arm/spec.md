## ADDED Requirements

### Requirement: Cross.toml SHALL define custom Docker configuration for Linux ARM

`Cross.toml` MUST 为 aarch64-unknown-linux-gnu target 指定自定义 Dockerfile 路径。

#### Scenario: Cross uses custom Dockerfile
- **WHEN** 使用 cross 构建 Linux ARM target
- **THEN** 使用自定义 Dockerfile 而非默认镜像

### Requirement: Custom Dockerfile SHALL include all Bevy system dependencies

自定义 Dockerfile MUST 基于 `ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main`，安装 arm64 架构的 `libasound2-dev`、`libudev-dev`、`libwayland-dev`、`libxkbcommon-dev`、`pkg-config`。

#### Scenario: Bevy dependencies available in cross container
- **WHEN** cross 执行 Linux ARM 构建
- **THEN** Docker 容器内已安装所有 Bevy 所需的 arm64 系统依赖

#### Scenario: pkg-config finds arm64 libraries
- **WHEN** 构建过程中 Bevy 通过 pkg-config 查找系统库
- **THEN** `PKG_CONFIG_PATH` 指向 arm64 架构的 pkgconfig 目录
