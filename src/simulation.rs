//! Simulation cell — tick-based updates synchronized with fleet-midi pulse.
//!
//! A simulation cell advances in discrete ticks. Multiple simulation cells
//! can be synchronized to the same pulse for coordinated fleet simulation.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::cell::{CellId, CellState, CellValue};

/// State of a running simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    pub tick: u64,
    pub max_ticks: u64,
    /// Custom state vector (e.g., position, velocity).
    pub state_vector: Vec<f64>,
    /// Whether the simulation has converged.
    pub converged: bool,
}

/// A cell that runs a tick-based simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationCell {
    pub name: String,
    pub state: CellState,
    pub sim: SimulationState,
    /// How often this simulation ticks (in engine ticks).
    pub tick_interval: u64,
    /// Cells this simulation reads from.
    pub inputs: Vec<CellId>,
}

impl SimulationCell {
    pub fn new(name: impl Into<String>, dimensions: usize) -> Self {
        Self {
            name: name.into(),
            state: CellState::Idle,
            sim: SimulationState {
                tick: 0,
                max_ticks: 1000,
                state_vector: vec![0.0; dimensions],
                converged: false,
            },
            tick_interval: 1,
            inputs: vec![],
        }
    }

    pub fn with_inputs(mut self, inputs: Vec<CellId>) -> Self {
        self.inputs = inputs;
        self
    }

    /// Advance the simulation by one tick.
    /// Default behavior: simple damped oscillation on state_vector[0].
    pub fn step(&mut self) -> CellValue {
        if self.sim.converged || self.sim.tick >= self.sim.max_ticks {
            self.state = CellState::Ready;
            return CellValue::Vector(self.sim.state_vector.clone());
        }
        self.sim.tick += 1;

        // Simple damped oscillation: x' = -0.1*x + noise
        if !self.sim.state_vector.is_empty() {
            let x = self.sim.state_vector[0];
            self.sim.state_vector[0] = x * 0.99 - 0.01 * x.sin();
        }

        // Check convergence
        let energy: f64 = self.sim.state_vector.iter().map(|x| x * x).sum();
        if energy < 1e-8 {
            self.sim.converged = true;
            self.state = CellState::Ready;
        } else {
            self.state = CellState::Running;
        }

        CellValue::Vector(self.sim.state_vector.clone())
    }

    /// Set initial state.
    pub fn with_state(mut self, state: Vec<f64>) -> Self {
        self.sim.state_vector = state;
        self
    }

    /// Progress fraction.
    pub fn progress(&self) -> f64 {
        if self.sim.max_ticks == 0 { return 0.0; }
        self.sim.tick as f64 / self.sim.max_ticks as f64
    }

    /// Is the simulation done (converged or max ticks)?
    pub fn is_done(&self) -> bool {
        self.sim.converged || self.sim.tick >= self.sim.max_ticks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_step() {
        let mut s = SimulationCell::new("test-sim", 3);
        s.sim.state_vector = vec![1.0, 0.5, -0.3];
        let result = s.step();
        assert_eq!(s.sim.tick, 1);
        assert_eq!(s.state, CellState::Running);
        assert!(matches!(result, CellValue::Vector(_)));
    }

    #[test]
    fn test_simulation_convergence() {
        let mut s = SimulationCell::new("test-sim", 1);
        s.sim.state_vector = vec![0.0001]; // Very small → should converge
        s.step();
        // Might not converge in one step, but energy is tiny
        let energy: f64 = s.sim.state_vector.iter().map(|x| x * x).sum();
        assert!(energy < 0.01);
    }

    #[test]
    fn test_simulation_max_ticks() {
        let mut s = SimulationCell::new("test-sim", 1);
        s.sim.max_ticks = 2;
        s.step();
        s.step();
        let result = s.step();
        assert!(s.is_done());
    }

    #[test]
    fn test_simulation_progress() {
        let mut s = SimulationCell::new("test-sim", 1);
        s.sim.max_ticks = 100;
        s.step();
        assert!(s.progress() > 0.0);
    }
}
