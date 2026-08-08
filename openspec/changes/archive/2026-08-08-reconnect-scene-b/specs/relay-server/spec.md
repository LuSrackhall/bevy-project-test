## ADDED Requirements

### Requirement: Relay re-sends GameStarted to reconnecting players in a started game

When a client joins a game that has already started AND its JoinGame reuses a `Disconnected` seat (i.e., it is a reconnecting player whose process restarted), the relay SHALL respond with `GameJoined`, then send `GameStarted` (with seed) again, before the reconnect pages. Clients already Playing SHALL ignore the duplicate `GameStarted` (only the lobby transition handles it).

#### Scenario: restarted process gets GameStarted on reconnect

- **WHEN** a player's process restarts, reconnects (JoinGame reuses the Disconnected seat) to a game that is already started
- **THEN** the relay SHALL send `GameStarted { seed, player_count }` to that player after `GameJoined`, so its lobby can transition to Playing and rebuild with the reconnect metadata seed

#### Scenario: duplicate GameStarted is harmless to live clients

- **WHEN** the relay re-sends `GameStarted` to a reconnecting player while other clients are Playing
- **THEN** the Playing clients SHALL ignore the duplicate (no state reset, no world rebuild)
