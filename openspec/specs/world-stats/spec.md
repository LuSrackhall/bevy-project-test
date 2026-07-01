## ADDED Requirements

### Requirement: Faction Counting

`simulation::world_stats::count_factions` SHALL return a `FactionCounts` containing per-faction soldier and city counts, using `BTreeMap<Faction, (u32, u32)>` for deterministic iteration order.

#### Scenario: Count after map generation
- **WHEN** `count_factions` is called on a world initialized with `MapSize::Small` and seed 42
- **THEN** the result MUST contain at least one unit (soldier or city) across all factions

#### Scenario: Deterministic across same-seed runs
- **WHEN** two worlds are initialized with the same seed (42) and same map size (Small)
- **THEN** `count_factions` SHALL return identical `FactionCounts` for both worlds

#### Scenario: Dynamic faction support
- **WHEN** only `Player` and `Enemy` factions have units
- **THEN** `count_factions().factions` SHALL contain exactly two entries
- **AND** factions without units SHALL NOT appear in the map

### Requirement: Per-Faction Accessors

`FactionCounts` SHALL provide `soldiers(faction)` and `cities(faction)` methods returning `u32`, defaulting to 0 for factions not present. It SHALL also provide `total_soldiers()` and `total_cities()` returning the sum across all factions.

#### Scenario: Accessor for missing faction
- **WHEN** `Faction::Neutral` has no units
- **THEN** `counts.soldiers(Faction::Neutral)` SHALL return 0
- **AND** `counts.cities(Faction::Neutral)` SHALL return 0
