## 1. 修复 create_bar 中子实体的 Visibility

- [x] 1.1 在 `create_bar` 中将 `let vis = ...` 改为 `let root_vis = ...`，仅用于根实体的 `spawn` 调用
- [x] 1.2 将 `with_children` 中所有子实体的 `vis` 替换为硬编码的 `Visibility::Inherited`

## 2. 验证

- [x] 2.1 `cargo check` 确认编译通过
- [ ] 2.2 运行游戏手动验证：切换显示模式后，城池和已有士兵的血条正确显示/隐藏
