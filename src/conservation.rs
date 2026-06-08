//! Conservation monitoring across the grid.
//!
//! Tracks γ + η = budget across all agent cells, computing fleet-wide
//! health and detecting violations before they cascade.

use crate::cell::{Cell, CellId, AgentCell};
use crate::grid::Grid;

/// Grid-wide conservation monitor.
#[derive(Debug, Clone)]
pub struct ConservationMonitor {
    /// Total budget for the entire grid.
    pub total_budget: f64,
    /// Per-cell tolerance.
    pub tolerance: f64,
    /// History of health readings (tick → health).
    pub history: Vec<(u64, f64)>,
}

impl ConservationMonitor {
    pub fn new(total_budget: f64, tolerance: f64) -> Self {
        Self { total_budget, tolerance, history: Vec::new() }
    }

    /// Compute fleet-wide conservation health (0.0–1.0).
    pub fn health(&self, grid: &Grid) -> f64 {
        let mut total_gamma = 0.0;
        let mut total_eta = 0.0;
        let mut agent_count = 0;

        for cell in grid.cells().map(|(_, c)| c) {
            if let Cell::Agent(a) = cell {
                total_gamma += a.gamma;
                total_eta += a.eta;
                agent_count += 1;
            }
        }

        if agent_count == 0 || self.total_budget < 1e-12 {
            return 1.0; // No agents = trivially healthy
        }

        let error = (total_gamma + total_eta - self.total_budget).abs();
        1.0 - (error / self.total_budget).min(1.0)
    }

    /// Record a health reading for a given tick.
    pub fn record(&mut self, tick: u64, grid: &Grid) -> f64 {
        let h = self.health(grid);
        self.history.push((tick, h));
        h
    }

    /// Number of agents violating their individual budgets.
    pub fn violations(&self, grid: &Grid) -> Vec<CellId> {
        grid.cells()
            .filter(|(_, c)| matches!(c, Cell::Agent(a) if !a.is_healthy(self.tolerance)))
            .map(|(id, _)| id)
            .collect()
    }

    /// Total γ across all agents.
    pub fn total_gamma(&self, grid: &Grid) -> f64 {
        grid.cells().filter_map(|(_, c)| {
            if let Cell::Agent(a) = c { Some(a.gamma) } else { None }
        }).sum()
    }

    /// Total η across all agents.
    pub fn total_eta(&self, grid: &Grid) -> f64 {
        grid.cells().filter_map(|(_, c)| {
            if let Cell::Agent(a) = c { Some(a.eta) } else { None }
        }).sum()
    }

    /// Whether the entire grid is conserving.
    pub fn is_healthy(&self, grid: &Grid) -> bool {
        self.health(grid) > 0.9 && self.violations(grid).is_empty()
    }

    /// Trend: is health improving or degrading over recent ticks?
    pub fn trend(&self) -> ConservationTrend {
        if self.history.len() < 2 {
            return ConservationTrend::Stable;
        }
        let recent = &self.history[self.history.len().saturating_sub(5)..];
        let first = recent.first().map(|(_, h)| *h).unwrap_or(1.0);
        let last = recent.last().map(|(_, h)| *h).unwrap_or(1.0);
        let diff = last - first;
        if diff > 0.05 { ConservationTrend::Improving }
        else if diff < -0.05 { ConservationTrend::Degrading }
        else { ConservationTrend::Stable }
    }
}

/// Trend direction for conservation health.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConservationTrend {
    Improving,
    Stable,
    Degrading,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid_with_agents(agents: &[(f64, f64, f64)]) -> Grid {
        let mut grid = Grid::new();
        for (i, &(gamma, eta, budget)) in agents.iter().enumerate() {
            let mut a = AgentCell::new(format!("agent-{}", i), budget);
            a.gamma = gamma;
            a.eta = eta;
            grid.insert(CellId::new(i as u32, 0), Cell::Agent(a));
        }
        grid
    }

    #[test]
    fn test_healthy_grid() {
        let grid = make_grid_with_agents(&[(0.3, 0.2, 0.5), (0.4, 0.1, 0.5)]);
        let mon = ConservationMonitor::new(1.0, 0.01);
        assert!(mon.health(&grid) > 0.5);
    }

    #[test]
    fn test_empty_grid_healthy() {
        let grid = Grid::new();
        let mon = ConservationMonitor::new(1.0, 0.01);
        assert_eq!(mon.health(&grid), 1.0);
    }

    #[test]
    fn test_violations() {
        let grid = make_grid_with_agents(&[(0.3, 0.2, 0.5), (0.8, 0.5, 0.5)]);
        let mon = ConservationMonitor::new(1.0, 0.01);
        let viols = mon.violations(&grid);
        assert!(!viols.is_empty(), "Second agent violates budget");
    }

    #[test]
    fn test_record_and_trend() {
        let mut grid = make_grid_with_agents(&[(0.3, 0.2, 0.5)]);
        let mut mon = ConservationMonitor::new(0.5, 0.01);
        mon.record(0, &grid);
        assert_eq!(mon.history.len(), 1);
        // Add more readings
        mon.record(1, &grid);
        mon.record(2, &grid);
        assert_eq!(mon.trend(), ConservationTrend::Stable);
    }

    #[test]
    fn test_totals() {
        let grid = make_grid_with_agents(&[(0.3, 0.2, 0.5), (0.4, 0.1, 0.5)]);
        let mon = ConservationMonitor::new(1.0, 0.01);
        assert!((mon.total_gamma(&grid) - 0.7).abs() < 0.01);
        assert!((mon.total_eta(&grid) - 0.3).abs() < 0.01);
    }
}
