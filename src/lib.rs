//! # spreadsheet-engine — Core Engine for Living AI Spreadsheets
//!
//! Every cell can be a value, an AI agent, a training job, a simulation,
//! an A2A endpoint, a MIDI generator, or an evolutionary formula.
//!
//! ## Quick Start
//!
//! ```
//! use spreadsheet_engine::{Grid, Engine, Cell, CellId, ValueCell};
//!
//! let mut grid = Grid::new();
//! grid.insert(CellId::new(0, 0), Cell::Value(ValueCell::from(42)));
//! grid.insert(CellId::new(0, 1), Cell::Value(ValueCell::from(8)));
//!
//! let mut engine = Engine::new(grid);
//! engine.tick().unwrap();
//! ```
//!
//! ## Cell Types
//!
//! | Type | Purpose | Example |
//! |------|---------|---------|
//! | `Value` | Plain data | `42`, `"hello"` |
//! | `Agent` | AI agent with capabilities | LLM cell, classifier |
//! | `Training` | Active ML training job | fine-tuning, RL loop |
//! | `Simulation` | Tick-based simulation | fleet-midi pulse |
//! | `A2A` | Agent-to-agent endpoint | discovers other cells |
//! | `Midi` | MIDI event generator | sonification |
//! | `Formula` | Evolutionary formula | `EVOLVE`, `PARETO`, `SPECIES` |

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

pub use a2a::{A2ABus, A2ACell, A2AMessage};
pub use cell::{Cell, CellId, CellResult, CellState, EvalContext, ValueCell, AgentCell};
pub use conservation::ConservationMonitor;
pub use engine::Engine;
pub use error::Error;
pub use formula::{FormulaCell, FormulaOp};
pub use grid::Grid;
pub use midi::MidiCell;
pub use simulation::{SimulationCell, SimulationState};
pub use training::{TrainingCell, TrainingState};
