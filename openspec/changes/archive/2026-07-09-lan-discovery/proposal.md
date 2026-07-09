## Why

当前只能手动输入 relay 地址联机，LAN 用户无法自动发现彼此。

## What Changes

UDP broadcast + scan for relay server discovery. Main menu shows discovered servers.

## Capabilities

- `lan-discovery`: LAN 局域网自动发现 + 服务器列表 UI

## Impact

- bevy_adapter: `LanDiscoveryPacket` + `LanDiscoveryListener`
- relay: UDP broadcast in `start_relay()`
- render_view: LAN server list UI + connection flow
EOF

mkdir -p openspec/changes/lan-discovery/specs/lan-discovery
cat > openspec/changes/lan-discovery/specs/lan-discovery/spec.md << 'ENDOFFILE'
## ADDED Requirements

### Requirement: UDP beacon broadcast

The relay SHALL broadcast UDP beacons on the LAN at regular intervals.

#### Scenario: Relay discovered on LAN
- **WHEN** a relay starts
- **THEN** it SHALL broadcast UDP beacons to `255.255.255.255` every 3 seconds

### Requirement: Client discovery and display

The client SHALL listen for UDP beacons on the main menu and display discovered servers.

#### Scenario: Server appears in list
- **WHEN** a UDP beacon is received
- **THEN** the server SHALL appear in the LAN server list
- **AND** clicking it SHALL start the game connection
EOF

cat > openspec/changes/lan-discovery/design.md << 'ENDOFFILE'
## Context

详见 brainstorm-spec.md。

## Decisions

### D1: 固定长度协议

9 bytes: magic + version + port + players + state. No bincode, no strings.

### D2: LanDiscoveryListener

Resource with Arc<Mutex<Vec<LanDiscoveryPacket>>>, tokio background thread.

### D3: UI

OnEnter(MainMenu) start, OnExit stop. 5s timeout, dedup by addr.
