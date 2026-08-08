## Context

**Bug（用户实测 rc17,Mac + Win 同局域网互不可见房间）**:主机创建房间后,另一台机器收不到任何房间广播。

**根因(已定位)**:
- `LanDiscoveryListener`(lan.rs:30)在 LanLobby 房间列表页绑定 `0.0.0.0:9876` 用于接收广播。
- `handle_create_room`(render_view lib.rs:717)先 `controller.create_session()` 启动 relay 线程,**此时仍在 LanLobby**,监听器持有 9876。
- relay 线程立即绑定 beacon socket `0.0.0.0:9876`(thread.rs:98)→ `EADDRINUSE` → 「beacon disabled」→ **主机从不广播房间**。
- OnExit(LanLobby) 到下一帧才释放监听器,为时已晚。
- 代码无 SO_REUSEADDR,第二个绑定必然失败。Mac/Win 对称受影响。

## Goals / Non-Goals

**Goals:**
- 修复主机 beacon 端口冲突:beacon 能正常广播,其他机器能发现房间
- 保持监听器(浏览)与 beacon(托管)共存的语义
- 跨平台(Mac/Win)一致

**Non-Goals:**
- 防火墙配置(用户侧,文档提示)
- 发现协议格式变更(bincode/relay_id 匹配不变)
- 子网广播优化(/24 推导等,当前非根因)

## Decisions

### D1: beacon 绑定临时端口,不争 9876(根因修复)

- **beacon socket 改绑 `0.0.0.0:0`**(临时端口)+ `set_broadcast(true)`,仍向 `<subnet_broadcast>:9876` / `255.255.255.255:9876` / `127.0.0.1:9876` 发送。
- **原理**:beacon 只需向目标端口 9876 发送,源端口无关紧要;监听器匹配房间靠包内 `relay_id`(不是源端口)。`0.0.0.0:0` 彻底避免与监听器争端口。
- 监听器(lan.rs)不变,仍绑 `0.0.0.0:9876` 接收。

### D2: 回环/子网发送路径不变

- 发送目标地址不变(子网广播 + 全局广播 + 回环),仅源绑定端口改变。

## Risks / Trade-offs

- [防火墙拦截入站 UDP 9876] → 用户侧放行(文档/提示);**加入连接还需放行 relay 临时端口**(代码评审警告,发现≠可连)
- [测试端口 9876 与环境冲突] → `start_on(port)` 测试隔离 + drain 轮询截止时间(测试评审)
- [手写 socket 复现假绿] → 测试走生产路径(ThreadRelayRuntime::start),断言 listener 收到包(测试评审 R1)
- [子网广播 /24 推导在非 /24 网络失败] → 既有行为,非本 change 范围

## Migration Plan

1. 测试先行:复现测试(listener 持 9876 时 beacon 绑定 0.0.0.0:0 成功并发送;listener 收到)→ 端到端
2. thread.rs beacon bind `0.0.0.0:0`
3. 回归:全仓 `cargo test` + `cargo check`
4. 回滚:分支级 revert
