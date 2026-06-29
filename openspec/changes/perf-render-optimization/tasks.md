## 1. InfoBar 脏标记

- [x] 1.1 定义 CachedState 结构体（hp_cur, hp_max, level, exp, shield_hp, shield_max）
- [x] 1.2 在 unit_info_bar_system 中添加 Local<HashMap<UnitId, CachedState>> 缓存
- [x] 1.3 在 update_bar 入口添加脏检查：比较缓存 vs 当前值，不变则跳过
- [x] 1.4 在 dead_ids 清理循环中同步清理缓存条目
- [x] 1.5 删除 debug_shape.rs 中未使用的 _positions HashMap

## 2. 视口裁剪

- [x] 2.1 计算 Camera viewport AABB（center ± window_size * scale / 2）
- [x] 2.2 在 unit_info_bar_system 的 per-unit 循环前添加 AABB 过滤
- [x] 2.3 在 draw_debug_shapes_system 的 per-unit 循环前添加 AABB 过滤
- [x] 2.4 添加全部可见时的短路优化

## 3. 默认 InfoBarMode=Selected

- [ ] 3.1 将 InfoBarMode 默认值从 Classic 改为 Selected
- [ ] 3.2 运行完整测试套件

---

## Post-Implementation Workflow

1. **Verify**: Run myspec-verify skill
2. **User Acceptance**: Present results, ask user to confirm
3. **Merge**: After user accepts, run myspec-merge skill
