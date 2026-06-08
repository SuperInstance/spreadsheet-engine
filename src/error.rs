//! Error types for the spreadsheet engine.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Cell not found: {0}")]
    CellNotFound(String),

    #[error("Evaluation cycle detected involving: {0}")]
    CycleDetected(String),

    #[error("Conservation violation: {0}")]
    ConservationViolation(String),

    #[error("Training error: {0}")]
    TrainingError(String),

    #[error("Simulation error: {0}")]
    SimulationError(String),

    #[error("A2A error: {0}")]
    A2AError(String),

    #[error("Formula error: {0}")]
    FormulaError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
