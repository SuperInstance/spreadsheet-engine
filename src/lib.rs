//! # spreadsheet-engine: The Living AI Spreadsheet
//!
//! An AI spreadsheet where every cell can be an agent, a training job, a simulation,
//! an A2A endpoint, or a MIDI generator — with conservation laws that keep the whole
//! system thermodynamically honest.
//!
//! ## Cell Types
//!
//! 7 first-class cell types: Value, Agent, Training, Simulation, A2A, MIDI, Formula.
//!
//! ## Conservation
//!
//! Agent cells obey γ (compute) + η (memory) = budget. The `ConservationMonitor`
//! tracks this fleet-wide, detecting budget leaks before they cascade.
//!
//! ## Quick Start
//!
//! ```
//! use spreadsheet_engine::{Grid, Engine, Cell, cell::{CellId, ValueCell}};
//!
//! let mut grid = Grid::new();
//! grid.insert(CellId::new(0, 0), Cell::Value(ValueCell::from(42)));
//!
//! let mut engine = Engine::new(grid);
//! engine.tick().unwrap();
//! assert_eq!(engine.value(&CellId::new(0, 0)), Some(&spreadsheet_engine::cell::CellValue::Number(42.0)));
//! ```

pub mod a2a;
pub mod cell;
pub mod conservation;
pub mod engine;
pub mod error;
pub mod formula;
pub mod grid;
pub mod midi;
pub mod simulation;
pub mod training;

pub use a2a::{A2ABus, A2ACell, A2AMessage, A2AMessageKind};
pub use cell::{
    Cell, CellId, CellResult, CellState, CellValue, EvalContext,
    AgentCell, ValueCell,
};
pub use conservation::{ConservationMonitor, ConservationTrend};
pub use engine::Engine;
pub use error::{Error, Result};
pub use formula::{FormulaCell, FormulaOp};
pub use grid::Grid;
pub use midi::MidiCell;
pub use simulation::SimulationCell;
pub use training::TrainingCell;
