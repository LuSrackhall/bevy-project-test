## Context

详见 brainstorm-spec.md。

## Goals / Non-Goals

**Goals:**
- 消除 Nagle 延迟
- 提高 beacon 跨网卡可达性

**Non-Goals:**
- 不降低 input_delay
- 不加依赖

## Decisions

### TCP_NODELAY

客户端 connect 后和 relay accept 后调用 `set_nodelay(true)`。

### 子网广播

`detect_lan_ip()` 已存在，推导 /24 广播地址，beacon 双发（255.255.255.255 + 子网广播）。

## Risks / Trade-offs

- [Risk] /24 假设 → 255.255.255.255 回退
