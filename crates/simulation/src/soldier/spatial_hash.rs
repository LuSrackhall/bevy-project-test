//! Spatial hash grid for deterministic O(k) neighbor queries.
//!
//! Uses BTreeMap for deterministic cell traversal order and UnitId sorting
//! within cells for deterministic intra-cell iteration (constitution §2.6).

use crate::types::*;
use std::collections::BTreeMap;

/// Entry in a spatial hash cell: position, collision radius, and owning UnitId.
#[derive(Clone, Copy)]
pub struct SpatialEntry {
    pub pos: FixedVec2,
    pub radius: u32,
    pub unit_id: UnitId,
}

/// A spatial hash grid storing SpatialEntry pairs.
/// BTreeMap ensures deterministic cell traversal; cells are sorted by UnitId.
pub struct SpatialHash {
    cell_size: i64,
    cells: BTreeMap<(i32, i32), Vec<SpatialEntry>>,
}

impl SpatialHash {
    pub fn new(cell_size: Fixed) -> Self {
        SpatialHash {
            cell_size: cell_size.0,
            cells: BTreeMap::new(),
        }
    }

    /// Insert an entry. Cell-internal Vec is kept sorted by UnitId for determinism.
    pub fn insert(&mut self, entry: SpatialEntry) {
        let key = self.cell_key(entry.pos);
        let cell = self.cells.entry(key).or_default();
        let pos = cell.partition_point(|e| e.unit_id < entry.unit_id);
        cell.insert(pos, entry);
    }

    /// Return all entries in the 9 cells around center.
    /// Traversal order: BTreeMap cell key order (deterministic), then UnitId-sorted within cell.
    pub fn query_nearby(&self, center: FixedVec2) -> Vec<SpatialEntry> {
        let (cx, cy) = self.cell_key(center);
        let mut result = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(cell) = self.cells.get(&(cx + dx, cy + dy)) {
                    result.extend_from_slice(cell);
                }
            }
        }
        result
    }

    /// Return all entries within `radius` of `center`, sweeping all cells
    /// that could contain entries within that distance.
    /// The number of cells swept is `(2 * ceil(radius / cell_size) + 1)^2`.
    /// Traversal order: BTreeMap cell key order (deterministic), then UnitId-sorted within cell.
    pub fn query_range(&self, center: FixedVec2, radius: i64) -> Vec<SpatialEntry> {
        let (cx, cy) = self.cell_key(center);
        // Number of cells to sweep in each direction from center cell
        let r = ((radius + self.cell_size - 1) / self.cell_size) as i32;
        let mut result = Vec::new();
        for dx in -r..=r {
            for dy in -r..=r {
                if let Some(cell) = self.cells.get(&(cx + dx, cy + dy)) {
                    result.extend_from_slice(cell);
                }
            }
        }
        result
    }

    fn cell_key(&self, pos: FixedVec2) -> (i32, i32) {
        let x = (pos.x.0 / self.cell_size) as i32;
        let y = (pos.y.0 / self.cell_size) as i32;
        (x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_nearby_returns_9_cells() {
        let mut hash = SpatialHash::new(Fixed::from_int(32));
        // Insert entries in 9 different cells around (0,0)
        for dx in -1..=1 {
            for dy in -1..=1 {
                hash.insert(SpatialEntry {
                    pos: FixedVec2::new(Fixed::from_int(dx * 32 + 16), Fixed::from_int(dy * 32 + 16)),
                    radius: 0,
                    unit_id: UnitId((dx + 2) as u64 * 10 + (dy + 2) as u64),
                });
            }
        }
        let result = hash.query_nearby(FixedVec2::new(Fixed::from_int(16), Fixed::from_int(16)));
        assert_eq!(result.len(), 9, "query_nearby should return entries from 9 cells");
    }

    #[test]
    fn test_query_range_small_radius_same_as_nearby() {
        let mut hash = SpatialHash::new(Fixed::from_int(32));
        for i in 0..10 {
            hash.insert(SpatialEntry {
                pos: FixedVec2::new(Fixed::from_int(i * 10), Fixed::ZERO),
                radius: 0,
                unit_id: UnitId(i as u64),
            });
        }
        // radius=30 with cell_size=32 → r=1, same as query_nearby (3x3=9 cells)
        let nearby = hash.query_nearby(FixedVec2::new(Fixed::from_int(16), Fixed::ZERO));
        let range = hash.query_range(FixedVec2::new(Fixed::from_int(16), Fixed::ZERO), 30);
        assert_eq!(nearby.len(), range.len(), "Small radius query_range should match query_nearby");
    }

    #[test]
    fn test_query_range_large_radius_sweeps_more_cells() {
        let mut hash = SpatialHash::new(Fixed::from_int(64));
        // Insert entries spread across many cells
        for i in 0..50 {
            hash.insert(SpatialEntry {
                pos: FixedVec2::new(Fixed::from_int(i * 64), Fixed::from_int(i * 64)),
                radius: 0,
                unit_id: UnitId(i as u64),
            });
        }
        // radius=200 with cell_size=64 → r=ceil(200/64)=4 → 9x9=81 cells
        let range = hash.query_range(FixedVec2::new(Fixed::from_int(32), Fixed::from_int(32)), 200);
        let nearby = hash.query_nearby(FixedVec2::new(Fixed::from_int(32), Fixed::from_int(32)));
        assert!(
            range.len() >= nearby.len(),
            "Large radius should sweep at least as many entries as nearby"
        );
    }

    #[test]
    fn test_query_range_deterministic() {
        let mut hash = SpatialHash::new(Fixed::from_int(32));
        for i in 0..20 {
            hash.insert(SpatialEntry {
                pos: FixedVec2::new(Fixed::from_int(i * 15), Fixed::from_int(i * 15)),
                radius: 0,
                unit_id: UnitId(i as u64),
            });
        }
        let center = FixedVec2::new(Fixed::from_int(100), Fixed::from_int(100));
        let r1 = hash.query_range(center, 100);
        let r2 = hash.query_range(center, 100);
        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.unit_id, b.unit_id, "query_range iteration order must be deterministic");
        }
    }
}
