## MODIFIED Requirements

### Requirement: Neutral city flips to attacker on capture

任何阵营的城市(含中立)被攻击且 HP ≤ 0 时,SHALL 翻转为**最后攻击者**的阵营(`last_attacker_faction`),而非硬编码的 0↔1 互换。单机 2 人下行为等价(攻击者只能是对方)。

#### Scenario: Player attacks neutral city to death

- **WHEN** 中立城池 HP 被玩家士兵攻击至 ≤ 0
- **THEN** 城池 faction 变为 Player(最后攻击者),HP 恢复为 20%,level 保持原值

#### Scenario: enemy city flips to its attacker in multiplayer

- **WHEN** 多人 FFA 中,玩家 C(非 0/1 阵营)的士兵将玩家 A 的城池 HP 攻击至 ≤ 0
- **THEN** 城池 faction 变为玩家 C 的阵营(最后攻击者),而非硬编码互换

#### Scenario: no attacker falls back to prior owner

- **WHEN** 城市 HP ≤ 0 但 `last_attacker_faction` 为 None(无攻击者记录)
- **THEN** 城池 faction 保持兜底行为(不因无攻击者而错变为固定阵营)
