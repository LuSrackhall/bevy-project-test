## ADDED Requirements

### Requirement: adaptive iteration exit

overlap_resolution_system SHALL exit iteration loop early when overlap_count is below a threshold. The comparison MUST use pure integer arithmetic (no floats).

#### Scenario: convergence reached
- **WHEN** overlap_count * 100 < total_unit_count
- **THEN** the iteration loop SHALL break

#### Scenario: not converged
- **WHEN** overlap_count * 100 >= total_unit_count
- **THEN** the next iteration SHALL run normally

#### Scenario: determinism preserved
- **WHEN** two runs reach the same overlap_count
- **THEN** they SHALL make the same exit decision
