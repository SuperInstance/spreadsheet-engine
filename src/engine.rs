//! Engine — the evaluation loop that drives the living spreadsheet.
//!
//! The engine ticks at a configurable rate, evaluating cells in dependency
//! order, routing A2A messages, and monitoring conservation constraints.

use std::collections::HashMap;
use std::time::Duration;
use crate::cell::EvalContext;
#[cfg(test)] use crate::cell::Cell;

use crate::a2a::A2ABus;
use crate::cell::{CellId, CellValue};
use crate::conservation::ConservationMonitor;
use crate::grid::Grid;

/// The core engine driving the living spreadsheet.
pub struct Engine {
    /// The grid of cells.
    pub grid: Grid,
    /// A2A message bus for inter-cell communication.
    pub a2a_bus: A2ABus,
    /// Conservation monitor.
    pub conservation: ConservationMonitor,
    /// Current tick counter.
    pub tick: u64,
    /// Tick rate for real-time simulation.
    pub tick_rate: Duration,
    /// Cached values from the last tick.
    values: HashMap<CellId, CellValue>,
}

impl Engine {
    /// Create a new engine around a grid.
    pub fn new(grid: Grid) -> Self {
        let budget = grid.total_budget;
        let tolerance = grid.tolerance;
        Self {
            grid,
            a2a_bus: A2ABus::new(),
            conservation: ConservationMonitor::new(budget, tolerance),
            tick: 0,
            tick_rate: Duration::from_millis(100),
            values: HashMap::new(),
        }
    }

    /// Set the tick rate for real-time updates.
    pub fn with_tick_rate(mut self, rate: Duration) -> Self {
        self.tick_rate = rate;
        self
    }

    /// Execute one tick: evaluate all cells in dependency order.
    pub fn tick(&mut self) -> crate::error::Result<()> {
        self.tick += 1;

        // Get evaluation order (topological sort)
        let order = self.grid.eval_order()
            .ok_or_else(|| crate::error::Error::CycleDetected("grid".into()))?;

        // Evaluate each cell
        for id in order {
            let result = {
                // Gather dependency values
                let dep_set = self.grid.dependencies_for(&id);
                let deps: HashMap<CellId, CellValue> = dep_set.iter()
                    .map(|dep| {
                        (*dep, self.values.get(dep).cloned().unwrap_or(CellValue::Empty))
                    }).collect();

                let ctx = EvalContext {
                    dependencies: deps,
                    tick: self.tick,
                    total_budget: self.grid.total_budget,
                };

                if let Some(cell) = self.grid.get(&id).cloned() {
                    engine_internal::evaluate_cell(cell, &ctx)
                } else {
                    continue;
                }
            };

            self.values.insert(id, result.value);
        }

        // Record conservation
        self.conservation.record(self.tick, &self.grid);

        Ok(())
    }

    /// Get the current value of a cell.
    pub fn value(&self, id: &CellId) -> Option<&CellValue> {
        self.values.get(id)
    }

    /// Get all current values.
    pub fn all_values(&self) -> &HashMap<CellId, CellValue> {
        &self.values
    }

    /// Run N ticks.
    pub fn run_ticks(&mut self, n: u64) -> crate::error::Result<()> {
        for _ in 0..n {
            self.tick()?;
        }
        Ok(())
    }

    /// Current grid health (0.0–1.0).
    pub fn health(&self) -> f64 {
        self.conservation.health(&self.grid)
    }

    /// Number of cells in the grid.
    pub fn cell_count(&self) -> usize {
        self.grid.len()
    }
}

// Internal module to keep evaluate_cell private
mod engine_internal {
    use crate::cell::*;
    

    pub fn evaluate_cell(cell: Cell, ctx: &EvalContext) -> CellResult {
        match cell {
            Cell::Value(v) => CellResult::ok(v.value),
            Cell::Agent(a) => {
                // Agent evaluation: compute fitness from capabilities
                let score: f64 = a.capabilities.values().sum::<f64>()
                    / a.capabilities.len().max(1) as f64;
                CellResult::ok(CellValue::Number(score))
            }
            Cell::Training(mut t) => {
                let val = t.step();
                CellResult::ok(val)
            }
            Cell::Simulation(mut s) => {
                let val = s.step();
                CellResult::ok(val)
            }
            Cell::A2A(a) => {
                CellResult::ok(a.last_value)
            }
            Cell::Midi(mut m) => {
                // MIDI cells sonify their first dependency
                let dep_val = ctx.dependencies.values().next().cloned().unwrap_or(CellValue::Number(0.0));
                m.sonify(&dep_val);
                CellResult::ok(CellValue::Text(format!("MIDI ch{}", m.channel)))
            }
            Cell::Formula(mut f) => {
                let input_values: Vec<CellValue> = f.inputs.iter()
                    .filter_map(|id| ctx.dependencies.get(id).cloned())
                    .collect();
                let result = f.evaluate(&input_values);
                CellResult::ok(result)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::ValueCell;

    #[test]
    fn test_engine_single_tick() {
        let mut grid = Grid::new();
        grid.insert(CellId::new(0, 0), Cell::Value(ValueCell::from(42)));
        let mut engine = Engine::new(grid);
        engine.tick().unwrap();
        assert_eq!(engine.value(&CellId::new(0, 0)), Some(&CellValue::Number(42.0)));
    }

    #[test]
    fn test_engine_dependency_order() {
        let mut grid = Grid::new();
        let a = CellId::new(0, 0);
        let b = CellId::new(0, 1);
        grid.insert(a, Cell::Value(ValueCell::from(10)));
        grid.insert(b, Cell::Value(ValueCell::from(20)));
        grid.add_dependency(b, a); // b depends on a
        let mut engine = Engine::new(grid);
        engine.tick().unwrap();
        assert!(engine.value(&a).is_some());
        assert!(engine.value(&b).is_some());
    }

    #[test]
    fn test_engine_run_ticks() {
        let mut grid = Grid::new();
        grid.insert(CellId::new(0, 0), Cell::Value(ValueCell::from(1)));
        let mut engine = Engine::new(grid);
        engine.run_ticks(10).unwrap();
        assert_eq!(engine.tick, 10);
    }

    #[test]
    fn test_engine_with_agent() {
        let mut grid = Grid::new();
        let mut agent = crate::cell::AgentCell::new("alice", 1.0);
        agent.capabilities.insert("rust".into(), 0.9);
        agent.capabilities.insert("python".into(), 0.7);
        grid.insert(CellId::new(0, 0), Cell::Agent(agent));
        let mut engine = Engine::new(grid);
        engine.tick().unwrap();
        let val = engine.value(&CellId::new(0, 0));
        assert!(matches!(val, Some(CellValue::Number(n)) if *n > 0.0));
    }

    #[test]
    fn test_engine_health() {
        let mut grid = Grid::new();
        grid.insert(CellId::new(0, 0), Cell::Value(ValueCell::from(1)));
        let mut engine = Engine::new(grid);
        engine.tick().unwrap();
        assert!(engine.health() > 0.0);
    }

    #[test]
    fn test_engine_training_cell() {
        let mut grid = Grid::new();
        grid.insert(CellId::new(0, 0), Cell::Training(crate::training::TrainingCell::new("model-x", 50)));
        let mut engine = Engine::new(grid);
        engine.tick().unwrap();
        let val = engine.value(&CellId::new(0, 0));
        assert!(val.is_some());
    }

    #[test]
    fn test_engine_simulation_cell() {
        let mut grid = Grid::new();
        let mut sim = crate::simulation::SimulationCell::new("test-sim", 2);
        sim = sim.with_state(vec![1.0, -0.5]);
        grid.insert(CellId::new(0, 0), Cell::Simulation(sim));
        let mut engine = Engine::new(grid);
        engine.run_ticks(5).unwrap();
        assert!(engine.tick >= 5);
    }

    #[test]
    fn test_engine_formula_cell() {
        let mut grid = Grid::new();
        let a = CellId::new(0, 0);
        let b = CellId::new(0, 1);
        let c = CellId::new(0, 2);
        grid.insert(a, Cell::Value(ValueCell::from(3)));
        grid.insert(b, Cell::Value(ValueCell::from(4)));
        let formula = crate::formula::FormulaCell::new(
            crate::formula::FormulaOp::Sum,
            vec![a, b],
        );
        grid.insert(c, Cell::Formula(formula));
        grid.add_dependency(c, a);
        grid.add_dependency(c, b);
        let mut engine = Engine::new(grid);
        engine.tick().unwrap();
        // Formula should evaluate to 3 + 4 = 7
        let val = engine.value(&c);
        assert!(matches!(val, Some(CellValue::Number(n)) if (*n - 7.0).abs() < 0.01), "Expected 7.0, got {:?}", val);
    }
}
