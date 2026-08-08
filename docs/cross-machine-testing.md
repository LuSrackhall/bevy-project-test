# 跨机器联机测试指南

## 前置条件

两（或多）台机器在同一局域网（或有路由可达），每台都下载或编译了同一版本的：

- `city-conquest` 游戏客户端
- `relay` 服务器二进制

> 如果是从 GitHub Releases 下载：解压后 `city-conquest/` 目录下同时包含两个二进制。

## 步骤

### 1. 获取机器 A 的内网 IP

```bash
# macOS
ipconfig getifaddr en0

# Linux
ip addr show | grep inet

# Windows (PowerShell)
ipconfig
```

记下输出的 IP，假设为 `192.168.1.100`。

### 2. 机器 A 启动 relay

```bash
# 终端 1：relay 服务器
./relay --port 9876 --seed 42 --players 2
```

或从源码：

```bash
cargo run -p relay -- --port 9876 --seed 42 --players 2
```

看到 `Relay on port 9876 (players=2, seed=42)` 即成功。

### 3. 机器 A 启动客户端

```bash
# 终端 2：Player 0（连接本地 relay）
./city-conquest --relay 127.0.0.1:9876 --player-id 0 --players 2 --windowed
```

或从源码：

```bash
cargo run -- --relay 127.0.0.1:9876 --player-id 0 --players 2 --windowed
```

### 4. 机器 B 启动客户端

```bash
# 连接机器 A 的 relay
./city-conquest --relay 192.168.1.100:9876 --player-id 1 --players 2 --windowed
```

> 可选 `--windowed` 避免全屏抢占焦点。

### 5. 验证

两台机器的客户端会自动进入 Lobby 并开始对局。等待游戏进入 Playing 状态后：

- 双方框选自己的单位（左键拖拽）
- 右键空地移动
- 观察日志：`[NET] driver: advancing tick X with Y commands` — `Y` 应为正数

### 日志查看

如果遇到问题，查看日志：

```bash
# 终端输出可 grep 过滤
./city-conquest --relay ... 2>&1 | grep -E '\[NET\]|error|Error'

# 或重定向到文件
./city-conquest --relay ... > /tmp/game.log 2>&1
```

关键日志标记：
- `[NET] driver: advancing tick X with Y commands` — tick 推进且含命令
- `[NET] poll: pushed tick X to relay_buffer` — 远程命令已到达本地
- `[LAN] bind failed` — 无害警告，仅表示局域网发现端口被占用

## 三人及以上

```bash
# relay 启动时指定人数
./relay --port 9876 --seed 42 --players 3

# 各客户端
./city-conquest --relay 192.168.1.100:9876 --player-id 0 --players 3
./city-conquest --relay 192.168.1.100:9876 --player-id 1 --players 3
./city-conquest --relay 192.168.1.100:9876 --player-id 2 --players 3
```

## 防火墙放行

房间发现与联机依赖 UDP 入站,需在每台机器的防火墙放行：

- **入站 UDP 9876** — 发现 beacon 广播目标端口(浏览房间列表的监听端口)
- **入站 UDP 临时端口(relay 实际绑定端口)** — 加入对局时连接 relay 游戏 socket(`[::]:0` 由 OS 分配)

macOS:「系统设置 → 网络 → 防火墙」允许应用接收传入连接;Windows:「Windows Defender 防火墙 → 允许应用通过」放行 `city-conquest.exe` 与 `relay.exe`(或放行对应端口)。

> 如果能看到房间但加入失败(无响应/超时),通常是 relay 临时端口未被放行;若完全看不到房间,检查 9876 入站是否被拦。
