//! Core types for the deterministic simulation.
//!
//! All spatial and game-state values use fixed-point arithmetic.
//! No floating-point types are allowed in simulation logic.

use bevy_ecs::prelude::{Component, Resource};
use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

// ═══════════════════════════════════════════════════════════════
// Fixed-point number: i64 with 8 fractional bits
// ═══════════════════════════════════════════════════════════════

/// Fixed-point number with 8 fractional bits.
/// Precision: 1/256 ≈ 0.0039
/// Range: ~±8.3 million internal units
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Serialize, Deserialize)]
pub struct Fixed(pub i64);

/// Number of fractional bits.
pub const FIXED_FRAC_BITS: i64 = 8;
/// One in fixed-point representation: 1.0 = 256
pub const FIXED_ONE: i64 = 256;

impl Fixed {
    pub const ZERO: Fixed = Fixed(0);
    pub const ONE: Fixed = Fixed(FIXED_ONE);

    /// Create a Fixed from an integer.
    #[inline]
    pub fn from_int(n: i32) -> Self {
        Fixed(n as i64 * FIXED_ONE)
    }

    /// Create a Fixed from a float (only for config/initialization).
    #[inline]
    pub fn from_float(f: f32) -> Self {
        Fixed((f * FIXED_ONE as f32) as i64)
    }

    /// Convert to float (only for presentation layer).
    #[inline]
    pub fn to_float(self) -> f32 {
        self.0 as f32 / FIXED_ONE as f32
    }

    /// Absolute value.
    #[inline]
    pub fn abs(self) -> Self {
        Fixed(self.0.abs())
    }

    /// Maximum of two values.
    #[inline]
    pub fn max(self, other: Fixed) -> Fixed {
        Fixed(self.0.max(other.0))
    }

    /// Minimum of two values.
    #[inline]
    pub fn min(self, other: Fixed) -> Fixed {
        Fixed(self.0.min(other.0))
    }
}

// ── Arithmetic ──

impl Add for Fixed {
    type Output = Fixed;
    #[inline]
    fn add(self, rhs: Fixed) -> Fixed {
        Fixed(self.0 + rhs.0)
    }
}

impl Sub for Fixed {
    type Output = Fixed;
    #[inline]
    fn sub(self, rhs: Fixed) -> Fixed {
        Fixed(self.0 - rhs.0)
    }
}

impl Mul for Fixed {
    type Output = Fixed;
    #[inline]
    fn mul(self, rhs: Fixed) -> Fixed {
        Fixed(((self.0 as i128 * rhs.0 as i128) >> FIXED_FRAC_BITS) as i64)
    }
}

impl Div for Fixed {
    type Output = Fixed;
    #[inline]
    fn div(self, rhs: Fixed) -> Fixed {
        if rhs.0 == 0 {
            Fixed(0)
        } else {
            Fixed(((self.0 as i128 * FIXED_ONE as i128) / rhs.0 as i128) as i64)
        }
    }
}

impl AddAssign for Fixed {
    #[inline]
    fn add_assign(&mut self, rhs: Fixed) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Fixed {
    #[inline]
    fn sub_assign(&mut self, rhs: Fixed) {
        self.0 -= rhs.0;
    }
}

impl MulAssign for Fixed {
    #[inline]
    fn mul_assign(&mut self, rhs: Fixed) {
        self.0 = ((self.0 as i128 * rhs.0 as i128) >> FIXED_FRAC_BITS) as i64;
    }
}

impl DivAssign for Fixed {
    #[inline]
    fn div_assign(&mut self, rhs: Fixed) {
        if rhs.0 != 0 {
            self.0 = ((self.0 as i128 * FIXED_ONE as i128) / rhs.0 as i128) as i64;
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// FixedVec2: 2D fixed-point vector
// ═══════════════════════════════════════════════════════════════

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub struct FixedVec2 {
    pub x: Fixed,
    pub y: Fixed,
}

impl FixedVec2 {
    pub const ZERO: FixedVec2 = FixedVec2 {
        x: Fixed::ZERO,
        y: Fixed::ZERO,
    };

    pub fn new(x: Fixed, y: Fixed) -> Self {
        FixedVec2 { x, y }
    }

    /// Squared length — use this for distance comparisons per Const. 7.1.
    /// NEVER compute sqrt for distance checks.
    #[inline]
    pub fn length_squared(self) -> Fixed {
        let x2 = self.x * self.x;
        let y2 = self.y * self.y;
        x2 + y2
    }

    /// Dot product.
    #[inline]
    pub fn dot(self, other: FixedVec2) -> Fixed {
        self.x * other.x + self.y * other.y
    }

    /// Component-wise maximum.
    #[inline]
    pub fn min(self, other: FixedVec2) -> FixedVec2 {
        FixedVec2 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    /// Component-wise minimum.
    #[inline]
    pub fn max(self, other: FixedVec2) -> FixedVec2 {
        FixedVec2 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
}

impl Add for FixedVec2 {
    type Output = FixedVec2;
    #[inline]
    fn add(self, rhs: FixedVec2) -> FixedVec2 {
        FixedVec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for FixedVec2 {
    type Output = FixedVec2;
    #[inline]
    fn sub(self, rhs: FixedVec2) -> FixedVec2 {
        FixedVec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<Fixed> for FixedVec2 {
    type Output = FixedVec2;
    #[inline]
    fn mul(self, rhs: Fixed) -> FixedVec2 {
        FixedVec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div<Fixed> for FixedVec2 {
    type Output = FixedVec2;
    #[inline]
    fn div(self, rhs: Fixed) -> FixedVec2 {
        FixedVec2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// UnitId: logical entity identifier
// ═══════════════════════════════════════════════════════════════

/// Globally unique identifier for simulation entities.
/// Must be used for all cross-entity references in simulation.
/// Must NOT be confused with Bevy's `Entity`.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Serialize, Deserialize)]
pub struct UnitId(pub u64);

/// Monotonically increasing ID generator.
#[derive(Resource, Default)]
pub struct IdGenerator(u64);

/// The seed used to initialize the simulation world.
/// Persisted for Replay reconstruction.
#[derive(Resource, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SimulationSeed(pub u64);

impl IdGenerator {
    pub fn new() -> Self {
        IdGenerator(0)
    }

    pub fn next_id(&mut self) -> UnitId {
        let id = UnitId(self.0);
        self.0 += 1;
        id
    }
}

// ═══════════════════════════════════════════════════════════════
// Slot / Controller / FactionId 所有权模型
// ═══════════════════════════════════════════════════════════════

/// 阵营 ID — 单位/城市属于哪个阵营。
/// Simulation 只认识这个 ID，不知道谁在控制。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub struct FactionId(pub u8);

/// 队伍 ID — 胜负关系分组。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub struct TeamId(pub u8);

/// 槽位编号 — 谁坐在哪个位置。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub struct SlotId(pub u8);

/// AI 难度配置（预留）。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiProfile(pub String);

impl Default for AiProfile {
    fn default() -> Self {
        Self("default".into())
    }
}

/// Agent ID（预留）。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct AgentId(pub u64);

/// 谁负责产生 Command。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Controller {
    /// 本地人类玩家。
    HumanLocal,
    /// 远程人类玩家（联机预留）。
    HumanRemote(SlotId),
    /// AI 控制。
    AI(AiProfile),
    /// LLM Agent 等未来扩展预留。
    Agent(AgentId),
    /// 回放数据源。
    Replay,
    /// 槽位关闭，不产生 Command。
    Disabled,
}

impl Controller {
    /// Disabled 返回 false，其余返回 true。
    pub fn is_active(&self) -> bool {
        !matches!(self, Controller::Disabled)
    }
}

/// 一个槽位的完整配置。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerSlot {
    pub slot_id: SlotId,
    pub controller: Controller,
    pub faction: FactionId,
    pub team: TeamId,
}

/// 当前 Session 的所有槽位分配。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Resource)]
pub struct PlayerSlots {
    pub slots: Vec<PlayerSlot>,
}

impl PlayerSlots {
    /// 单人模式默认：1 × HumanLocal + 1 × AI。
    pub fn single_player() -> Self {
        Self {
            slots: vec![
                PlayerSlot {
                    slot_id: SlotId(0),
                    controller: Controller::HumanLocal,
                    faction: FactionId(0),
                    team: TeamId(0),
                },
                PlayerSlot {
                    slot_id: SlotId(1),
                    controller: Controller::AI(AiProfile::default()),
                    faction: FactionId(1),
                    team: TeamId(1),
                },
            ],
        }
    }

    /// Create N-player FFA slot configuration.
    pub fn multi_player(count: u8, local_player_id: u8) -> Self {
        assert!(count <= 8, "max 8 players");
        let slots = (0..count).map(|i| PlayerSlot {
            slot_id: SlotId(i),
            controller: if i == local_player_id {
                Controller::HumanLocal
            } else {
                Controller::HumanRemote(SlotId(i))
            },
            faction: FactionId(i),
            team: TeamId(0),
        }).collect();
        Self { slots }
    }

}

/// Bevy ECS Resource stored in the simulation world indicating which player_id
/// is the local human player. Defaults to 0 (Player faction) for single-player.
/// Set during game init for network mode.
#[derive(Clone, Copy, Debug, Resource)]
pub struct LocalPlayerId(pub u8);

impl Default for LocalPlayerId {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum SoldierType {
    Militia,
    Infantry,
    Archer,
    Cavalry,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum SoldierState {
    Moving,
    Fighting,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum ShieldState {
    Normal,
    Blocking,
}

/// Shield item with independent HP. Can be held by a soldier or dropped on ground.
#[derive(Component, Clone, Debug)]
pub struct ShieldItem {
    pub hp: u32,
    pub max_hp: u32,
}

/// A shield dropped on the ground after a soldier dies.
#[derive(Component, Clone, Debug)]
pub struct DroppedShield {
    pub shield: ShieldItem,
    pub position: FixedVec2,
    pub drop_tick: u32,
    pub owner_faction: Option<FactionId>,
}

// ═══════════════════════════════════════════════════════════════
// Facing direction & attack windup
// ═══════════════════════════════════════════════════════════════

/// Unit facing direction — independent state updated by movement and attack targeting.
/// All 360 degrees represented as fixed-point. 0° = right, 90° = up, etc.
#[derive(Component, Clone, Debug)]
pub struct FacingDirection {
    pub angle: Fixed, // 0.0 to <360.0 in fixed-point degrees
}

/// Attack windup state — non-cavalry units pause briefly before attacking.
#[derive(Component, Clone, Debug)]
pub struct AttackWindup {
    pub remaining_ticks: u32,   // ticks remaining in windup; 0 = not winding up
    pub target: Option<UnitId>, // target to attack when windup completes
}

// ═══════════════════════════════════════════════════════════════
// Deterministic PRNG
// ═══════════════════════════════════════════════════════════════

/// Deterministic random number generator.
/// Uses SmallRng with a fixed seed for reproducible simulations.
#[derive(Resource)]
pub struct DeterministicRng(pub SmallRng);

impl DeterministicRng {
    /// Create a new deterministic RNG with the given seed.
    pub fn new(seed: u64) -> Self {
        DeterministicRng(SmallRng::seed_from_u64(seed))
    }

    /// Generate a random u64.
    pub fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    /// Generate a deterministic probability as permyriad (万分比).
    /// Returns u32 in [0, 10000] representing 0.00% to 100.00%.
    pub fn gen_probability_permyriad(&mut self) -> u32 {
        (self.0.next_u64() % 10001) as u32
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_from_int() {
        assert_eq!(Fixed::from_int(5), Fixed(5 * 256));
        assert_eq!(Fixed::from_int(0), Fixed(0));
        assert_eq!(Fixed::from_int(-3), Fixed(-3 * 256));
    }

    #[test]
    fn test_fixed_add() {
        let a = Fixed::from_int(3);
        let b = Fixed::from_int(5);
        assert_eq!(a + b, Fixed::from_int(8));
    }

    #[test]
    fn test_fixed_sub() {
        let a = Fixed::from_int(10);
        let b = Fixed::from_int(3);
        assert_eq!(a - b, Fixed::from_int(7));
    }

    #[test]
    fn test_fixed_mul() {
        let a = Fixed::from_int(3);
        let b = Fixed::from_int(2);
        assert_eq!(a * b, Fixed::from_int(6));
    }

    #[test]
    fn test_fixed_mul_fractional() {
        let a = Fixed::from_int(3);
        let half = Fixed(FIXED_ONE / 2); // 0.5
        let result = a * half;
        // 3 * 0.5 = 1.5 → 1.5 * 256 = 384
        // Tolerance: ±1 internal unit
        assert!((result.0 - 384).abs() <= 1, "got {}", result.0);
    }

    #[test]
    fn test_fixed_div() {
        let a = Fixed::from_int(10);
        let b = Fixed::from_int(2);
        assert_eq!(a / b, Fixed::from_int(5));
    }

    #[test]
    fn test_fixed_div_by_zero() {
        let a = Fixed::from_int(5);
        let b = Fixed::ZERO;
        assert_eq!(a / b, Fixed::ZERO); // safe division
    }

    #[test]
    fn test_fixed_vec2_length_squared() {
        let v = FixedVec2::new(Fixed::from_int(3), Fixed::from_int(4));
        let len_sq = v.length_squared();
        assert_eq!(len_sq, Fixed::from_int(25)); // 9 + 16
    }

    #[test]
    fn test_fixed_vec2_add() {
        let a = FixedVec2::new(Fixed::from_int(1), Fixed::from_int(2));
        let b = FixedVec2::new(Fixed::from_int(3), Fixed::from_int(4));
        let c = a + b;
        assert_eq!(c.x, Fixed::from_int(4));
        assert_eq!(c.y, Fixed::from_int(6));
    }

    #[test]
    fn test_fixed_vec2_sub() {
        let a = FixedVec2::new(Fixed::from_int(5), Fixed::from_int(8));
        let b = FixedVec2::new(Fixed::from_int(2), Fixed::from_int(3));
        let c = a - b;
        assert_eq!(c.x, Fixed::from_int(3));
        assert_eq!(c.y, Fixed::from_int(5));
    }

    #[test]
    fn test_unit_id_unique() {
        let mut gen = IdGenerator::new();
        let a = gen.next_id();
        let b = gen.next_id();
        let c = gen.next_id();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    #[test]
    fn test_unit_id_ordering() {
        let mut gen = IdGenerator::new();
        let a = gen.next_id();
        let b = gen.next_id();
        assert!(a < b);
    }

    #[test]
    fn test_deterministic_rng_same_seed_same_output() {
        let mut rng1 = DeterministicRng::new(42);
        let mut rng2 = DeterministicRng::new(42);
        let v1: Vec<u64> = (0..100).map(|_| rng1.next_u64()).collect();
        let v2: Vec<u64> = (0..100).map(|_| rng2.next_u64()).collect();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_deterministic_rng_different_seeds_different_output() {
        let mut rng1 = DeterministicRng::new(42);
        let mut rng2 = DeterministicRng::new(99);
        let v1: Vec<u64> = (0..10).map(|_| rng1.next_u64()).collect();
        let v2: Vec<u64> = (0..10).map(|_| rng2.next_u64()).collect();
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_squared_distance_vs_linear() {
        // Verify that squared distance comparison produces same ordering as linear distance
        let a = FixedVec2::new(Fixed::from_int(0), Fixed::from_int(0));
        let b = FixedVec2::new(Fixed::from_int(3), Fixed::from_int(4)); // distance 5
        let c = FixedVec2::new(Fixed::from_int(6), Fixed::from_int(8)); // distance 10

        let d_ab_sq = (b - a).length_squared();
        let d_ac_sq = (c - a).length_squared();

        assert!(d_ab_sq < d_ac_sq); // 25 < 100
    }

    #[test]
    fn test_multi_player_slots_3() {
        let slots = PlayerSlots::multi_player(3, 0);
        assert_eq!(slots.slots.len(), 3);
        assert_eq!(slots.slots[0].faction, FactionId(0));
        assert!(matches!(slots.slots[0].controller, Controller::HumanLocal));
        assert!(matches!(slots.slots[1].controller, Controller::HumanRemote(_)));
        assert!(matches!(slots.slots[2].controller, Controller::HumanRemote(_)));
    }

    #[test]
    fn test_multi_player_slots_4() {
        let slots = PlayerSlots::multi_player(4, 2);
        assert_eq!(slots.slots.len(), 4);
        assert_eq!(slots.slots[2].faction, FactionId(2));
        assert!(matches!(slots.slots[2].controller, Controller::HumanLocal));
        for i in 0..4 {
            assert_eq!(slots.slots[i as usize].faction, FactionId(i));
        }
    }

}
