## ADDED Requirements

### Requirement: MapBounds 资源
bevy_adapter SHALL 提供 `MapBounds { width: f32, height: f32 }` 资源。在 `reset_game_system` 完成后从 `MapGenConfig` 创建。

#### Scenario: MapBounds 初始化
- **WHEN** 地图生成完成，MapGenConfig 为 5000x5000
- **THEN** `MapBounds { width: 5000.0, height: 5000.0 }` 被插入 Bevy world
