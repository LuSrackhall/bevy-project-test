# ADR-002: Action 排序标签使用 const fn 而非 repr(u8)

## 状态

Accepted

## 决策

`Action` 枚举的确定性排序使用 `fn sort_tag(&self) -> u8` const 方法，而非 `#[repr(u8)]` 显式判别值。

## 理由

Rust 的 `#[repr(u8)]` 显式判别值只能用于**无字段枚举**。`Action` 的 `MoveTo`、`Attack`、`SetSeekStance` 等变体携带数据，无法标注 `= N` 判别值。

```rust
// 不能编译
#[repr(u8)]
pub enum Action {
    MoveTo { unit: UnitId, target: FixedVec2 } = 1, // ❌ 编译错误
}
```

`sort_tag()` 通过 `const fn` + `match` 达到相同效果：显式硬编码排序值，不依赖 Rust 枚举隐式判别值。

## 放弃的方案

1. **#[repr(u8)] + 无字段标签枚举**：需要额外定义一个 `ActionTag` 枚举并维护映射关系，增加复杂度。
2. **依赖 Rust 隐式判别值**：跨编译环境（LTO、profile-guided 优化）可能不一致。
3. **std::mem::discriminant**：返回 `Discriminant<Action>`，不保证可比较大小。

## 代价

新增变体时必须手动分配 `sort_tag` 值，不能依赖编译器自动赋值。

## 修改条件

若 Rust 未来支持带字段枚举的显式判别值，可迁移到 `#[repr(u8)]`。
