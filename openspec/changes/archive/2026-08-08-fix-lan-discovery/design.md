## Context

用户实测 Mac+Win 同局域网互不可见房间。根因：`handle_create_room`（render_view lib.rs:717）先 `create_session` 启动 relay 线程（仍在 LanLobby），relay 线程绑定 beacon socket `0.0.0.0:9876`（thread.rs:98）时被 `LanDiscoveryListener`（lan.rs:30，浏览时绑 0.0.0.0:9876）占用 → `EADDRINUSE` → beacon 禁用 → 主机从不广播。OnExit(LanLobby) 下一帧才释放 listener，为时已晚。无 SO_REUSEADDR。代码评审/宪法/测试评审均已确认根因与修复方向。

## Goals / Non-Goals

**Goals:**
- 修复 beacon 端口冲突，使主机能正常广播房间
- 测试走生产路径（非手写 socket 假绿）
- 跨平台（Mac/Win/CI matrix）一致

**Non-Goals:**
- 防火墙配置（用户侧，文档提示）
- 发现协议格式变更；/24 子网推导优化（既有行为）
- simulation 改动

## Decisions

### D1: beacon 绑临时端口（根因修复）

- thread.rs beacon bind `0.0.0.0:9876` → `0.0.0.0:0`，保留 `set_broadcast(true)`。
- 原理：UDP 按目的 `:9876` 递送，源端口无关；监听器按包内 `relay_id` 去重（lan.rs:48），不碰源端口。
- 选临时端口而非 SO_REUSEADDR：macOS 需 SO_REUSEPORT、Windows 语义不同，临时端口可移植。

### D2: 测试必须走生产路径（测试评审 R1）

- **回归 e2e**：`LanDiscoveryListener`（绑 0.0.0.0:9876）运行中 → `ThreadRelayRuntime::start`（生产 beacon）→ 轮询 `drain()`（≤5s）断言收到包且 `relay_id` 匹配。**预修必挂、修复后过**——若手写 socket 复现，beacon 回退 0.0.0.0:9876 时仍假绿。
- **断言落在「listener 收到包」而非「beacon 绑定成功」**：beacon bind 失败仅 eprintln 后置 None，`start()` 仍返回 Ok。
- **源端口探针（R2）**：独立探针 socket 断言源端口 ≠ 9876（lan.rs 的 recv_from 丢弃源地址）。
- **packet 单测（R3）**：encode/decode round-trip + 垃圾/错误 magic → None。

### D3: 测试隔离（测试评审端口安全）

- `LanDiscoveryListener` 增 `start_on(port)`（默认 9876）；回归测试用 9876（复现冲突），其余用临时端口；或文件内 Mutex 串行化。
- `drain()` 轮询 + 截止时间，禁止单次 sleep。
- beacon interval 参数化（测试用短值，生产 3s）。

### D4: 宪法

- 全在 bevy_adapter；beacon/listener 属传输/发现层，无仿真业务规则；无确定性影响（源端口不进包负载，去重按 relay_id）；无 simulation 白名单风险。

## Risks / Trade-offs

- [防火墙拦入站 UDP 9876] → 用户侧放行（文档提示）；临时 relay 端口也需放行（加入连接）
- [/24 推导在非 /24 网络失败] → 既有行为，Non-Goal
- [测试端口冲突] → D3 start_on(port) / Mutex
- [255.255.255.255 在 Windows 不可靠] → 测试断言只用 127.0.0.1 送达

## Migration Plan

1. 测试先行：packet 单测（R3）+ start_on(port) API + 生产路径 e2e（R1，预修必挂）
2. thread.rs beacon bind `0.0.0.0:0`
3. 回归：全仓 `cargo test` + `cargo check`
4. 回滚：分支级 revert

## Open Questions

- 防火墙文档放行指引的具体描述位置
- start_on(port) 的默认值（9876）与调用方接线
