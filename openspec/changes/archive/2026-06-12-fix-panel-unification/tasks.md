## 1. 布局重构

- [x] 1.1 底部改为二区：左 30%（信息面板）+ 右 70%（命令卡片+图鉴）
- [x] 1.2 左区同时 spawn 士兵面板和城池面板，通过 Display 切换
- [x] 1.3 移除中区城池面板、移除右区小地图占位、移除 `CityPanelRoot` 可见性查询

## 2. 更新逻辑

- [x] 2.1 `update_bottom_panel`: 城池选中 → 显示城池面板, 隐藏士兵面板；兵种选中 → 反之
- [x] 2.2 移除 `ParamSet` 中多余的 `city_root_q` 查询（面板直接通过 Node 切换 Display）

## 3. 验证

- [x] 3.1 `cargo check` + `cargo test` 通过
