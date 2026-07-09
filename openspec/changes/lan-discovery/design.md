## Context

详见 brainstorm-spec.md。

## Decisions

### D1: 固定长度协议

9 bytes: magic + version + port + players + state. No bincode, no strings.

### D2: LanDiscoveryListener

Resource with Arc<Mutex<Vec<LanDiscoveryPacket>>>, tokio background thread.

### D3: UI

OnEnter(MainMenu) start, OnExit stop. 5s timeout, dedup by addr.
