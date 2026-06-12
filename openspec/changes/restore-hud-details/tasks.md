## 1. 恢复城池面板信息

- [x] 1.1 恢复城池面板等级显示：`set_text(&mut tq, ht.c_info, &format!("[城池] Lv.{} (最高Lv.{})", city.level, city.max_level))`
- [x] 1.2 恢复城池面板 HP 文本和填充条：`set_text(&mut tq, ht.c_hp_text, &format!("HP {}/{}", ...))` + 填充条宽度和颜色更新
- [x] 1.3 恢复城池面板经验显示：`set_text(&mut tq, ht.c_exp, &format!("经验 {}/{}", ...))`

## 2. 恢复士兵面板信息

- [x] 2.1 恢复士兵面板查询中的 `Health` 和 `Level` 组件读取
- [x] 2.2 恢复单选士兵的 HP 文本、HP 填充条、EXP 文本、EXP 填充条、等级显示
- [x] 2.3 恢复多选士兵的汇总 HP 显示
- [x] 2.4 移除 "(HP/经验见头顶)" 提示文字

## 3. 验证

- [x] 3.1 `cargo check` 确认编译通过
- [ ] 3.2 运行游戏手动验证：选中单位后底部面板显示完整的 HP/EXP/等级信息
