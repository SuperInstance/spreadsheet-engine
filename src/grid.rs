//! Grid — the spreadsheet data structure.
//!
//! A `Grid` holds cells in a sparse HashMap keyed by [`CellId`].
//! It tracks dependencies between cells for correct evaluation order.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::cell::{Cell, CellId, CellValue, CellState};

/// The living spreadsheet grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    /// Sparse cell storage.
    cells: HashMap<CellId, Cell>,
    /// Dependency edges: cell → cells it depends on.
    dependencies: HashMap<CellId, HashSet<CellId>>,
    /// Reverse edges: cell → cells that depend on it (for propagation).
    dependents: HashMap<CellId, HashSet<CellId>>,
    /// Grid-wide conservation budget (γ + η ≤ budget for all agent cells).
    pub total_budget: f64,
    /// Conservation tolerance.
    pub tolerance: f64,
}

impl Grid {
    /// Create an empty grid.
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
            total_budget: 10.0,
            tolerance: 0.01,
        }
    }

    /// Create a grid with a specific conservation budget.
    pub fn with_budget(total_budget: f64, tolerance: f64) -> Self {
        Self { total_budget, tolerance, ..Self::new() }
    }

    /// Insert or replace a cell.
    pub fn insert(&mut self, id: CellId, cell: Cell) {
        self.cells.insert(id, cell);
    }

    /// Remove a cell and all its dependency edges.
    pub fn remove(&mut self, id: &CellId) -> Option<Cell> {
        // Clean up dependency edges
        if let Some(deps) = self.dependencies.remove(id) {
            for dep in deps {
                if let Some(set) = self.dependents.get_mut(&dep) {
                    set.remove(id);
                }
            }
        }
        if let Some(set) = self.dependents.remove(id) {
            for dependent in set {
                if let Some(deps) = self.dependencies.get_mut(&dependent) {
                    deps.remove(id);
                }
            }
        }
        self.cells.remove(id)
    }

    /// Get a cell by ID.
    pub fn get(&self, id: &CellId) -> Option<&Cell> {
        self.cells.get(id)
    }

    /// Get a mutable reference to a cell.
    pub fn get_mut(&mut self, id: &CellId) -> Option<&mut Cell> {
        self.cells.get_mut(id)
    }

    /// Get dependencies of a cell (public accessor).
    pub fn dependencies_for(&self, id: &CellId) -> HashSet<CellId> {
        self.dependencies.get(id).cloned().unwrap_or_default()
    }

    /// Add a dependency: `cell` depends on `on`.
    pub fn add_dependency(&mut self, cell: CellId, on: CellId) {
        self.dependencies.entry(cell).or_default().insert(on);
        self.dependents.entry(on).or_default().insert(cell);
    }

    /// Get cells that depend on the given cell.
    pub fn dependents_of(&self, id: &CellId) -> Vec<CellId> {
        self.dependents.get(id).map(|s| s.iter().copied().collect()).unwrap_or_default()
    }

    /// Topological sort of all cells respecting dependencies.
    /// Returns `None` if a cycle is detected.
    pub fn eval_order(&self) -> Option<Vec<CellId>> {
        let mut in_degree: HashMap<CellId, usize> = self.cells.keys()
            .map(|id| (*id, 0))
            .collect();

        for (cell_id, deps) in &self.dependencies {
            for dep in deps {
                // cell_id depends on dep → dep must come before cell_id
                *in_degree.entry(*dep).or_insert(0) += 0; // ensure exists
                if let Some(deg) = in_degree.get_mut(cell_id) {
                    *deg += 1;
                }
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<CellId> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut result = Vec::new();
        while let Some(id) = queue.pop_front() {
            result.push(id);
            // id is done → decrement in-degree of cells that depend on id
            if let Some(deps) = self.dependents.get(&id) {
                for &dep in deps {
                    if let Some(deg) = in_degree.get_mut(&dep) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dep);
                        }
                    }
                }
            }
        }

        if result.len() == self.cells.len() {
            Some(result)
        } else {
            None // cycle
        }
    }

    /// Number of cells in the grid.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Is the grid empty?
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Iterate over all cell IDs.
    pub fn cell_ids(&self) -> impl Iterator<Item = CellId> + '_ {
        self.cells.keys().copied()
    }

    /// Iterate over all cells.
    pub fn cells(&self) -> impl Iterator<Item = (CellId, &Cell)> {
        self.cells.iter().map(|(id, cell)| (*id, cell))
    }

    /// Count cells by type.
    pub fn type_counts(&self) -> HashMap<&'static str, usize> {
        let mut counts = HashMap::new();
        for cell in self.cells.values() {
            *counts.entry(cell.type_name()).or_insert(0) += 1;
        }
        counts
    }

    /// Check if adding "cell depends on on" would create a cycle.
    /// DFS from `on` through its dependencies, looking for `cell`.
    pub fn would_create_cycle(&self, cell: CellId, on: CellId) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![on];
        while let Some(current) = stack.pop() {
            if current == cell {
                return true;
            }
            if visited.insert(current) {
                if let Some(deps) = self.dependencies.get(&current) {
                    for dep in deps {
                        stack.push(*dep);
                    }
                }
            }
        }
        false
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::ValueCell;

    #[test]
    fn test_insert_and_get() {
        let mut grid = Grid::new();
        let id = CellId::new(0, 0);
        grid.insert(id, Cell::Value(ValueCell::from(42)));
        assert!(grid.get(&id).is_some());
        assert_eq!(grid.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut grid = Grid::new();
        let id = CellId::new(0, 0);
        grid.insert(id, Cell::Value(ValueCell::from(1)));
        assert!(grid.remove(&id).is_some());
        assert!(grid.is_empty());
    }

    #[test]
    fn test_dependency_tracking() {
        let mut grid = Grid::new();
        let a = CellId::new(0, 0);
        let b = CellId::new(0, 1);
        let c = CellId::new(0, 2);
        grid.insert(a, Cell::Value(ValueCell::from(1)));
        grid.insert(b, Cell::Value(ValueCell::from(2)));
        grid.insert(c, Cell::Value(ValueCell::from(3)));
        grid.add_dependency(c, a); // c depends on a
        grid.add_dependency(c, b); // c depends on b
        let deps = grid.dependents_of(&a);
        assert!(deps.contains(&c));
    }

    #[test]
    fn test_eval_order_linear() {
        let mut grid = Grid::new();
        let a = CellId::new(0, 0);
        let b = CellId::new(0, 1);
        let c = CellId::new(0, 2);
        grid.insert(a, Cell::Value(ValueCell::from(1)));
        grid.insert(b, Cell::Value(ValueCell::from(2)));
        grid.insert(c, Cell::Value(ValueCell::from(3)));
        grid.add_dependency(b, a); // b depends on a
        grid.add_dependency(c, b); // c depends on b
        let order = grid.eval_order().unwrap();
        assert!(order.iter().position(|x| *x == a) < order.iter().position(|x| *x == b));
        assert!(order.iter().position(|x| *x == b) < order.iter().position(|x| *x == c));
    }

    #[test]
    fn test_eval_order_cycle() {
        let mut grid = Grid::new();
        let a = CellId::new(0, 0);
        let b = CellId::new(0, 1);
        grid.insert(a, Cell::Value(ValueCell::from(1)));
        grid.insert(b, Cell::Value(ValueCell::from(2)));
        grid.add_dependency(a, b);
        grid.add_dependency(b, a);
        assert!(grid.eval_order().is_none());
    }

    #[test]
    fn test_type_counts() {
        let mut grid = Grid::new();
        grid.insert(CellId::new(0, 0), Cell::Value(ValueCell::from(1)));
        grid.insert(CellId::new(0, 1), Cell::Value(ValueCell::from(2)));
        grid.insert(CellId::new(1, 0), Cell::Agent(crate::cell::AgentCell::new("a", 1.0)));
        let counts = grid.type_counts();
        assert_eq!(counts["Value"], 2);
        assert_eq!(counts["Agent"], 1);
    }

    #[test]
    fn test_would_create_cycle() {
        let mut grid = Grid::new();
        let a = CellId::new(0, 0);
        let b = CellId::new(0, 1);
        let c = CellId::new(0, 2);
        grid.insert(a, Cell::Value(ValueCell::from(1)));
        grid.insert(b, Cell::Value(ValueCell::from(2)));
        grid.insert(c, Cell::Value(ValueCell::from(3)));
        grid.add_dependency(b, a); // b depends on a
        grid.add_dependency(c, b); // c depends on b
        // Adding a→c (a depends on c) would create cycle: a→c→b→a
        assert!(grid.would_create_cycle(a, c));
        // Adding c→d is fine
        assert!(!grid.would_create_cycle(c, CellId::new(99, 99)));
    }
}
