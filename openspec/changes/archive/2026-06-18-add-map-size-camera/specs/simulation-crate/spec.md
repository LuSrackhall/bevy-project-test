## MODIFIED Requirements

### Requirement: Map 生成
`generate_map` SHALL 接受 `MapSize` 参数，根据 MapSize 加载对应配置文件。城池数量改为密度驱动计算。

#### Scenario: generate_map 签名
- **WHEN** 调用 `generate_map(world, MapSize::Large)`
- **THEN** 从 `content/map/large.ron` 加载配置，按密度计算城池数并生成
