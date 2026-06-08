//! Training cell — manages an ML training job within the grid.
//!
//! A training cell represents an active training loop that can be started,
//! paused, and monitored. It tracks epochs, loss, and checkpoints.

use serde::{Deserialize, Serialize};

use crate::cell::{CellId, CellState, CellValue};

/// State of a training job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingState {
    pub current_epoch: u32,
    pub total_epochs: u32,
    pub loss: f64,
    pub best_loss: f64,
    pub learning_rate: f64,
}

impl Default for TrainingState {
    fn default() -> Self {
        Self { current_epoch: 0, total_epochs: 100, loss: f64::MAX, best_loss: f64::MAX, learning_rate: 0.001 }
    }
}

/// A cell that runs an ML training job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingCell {
    pub model_name: String,
    pub state: CellState,
    pub training: TrainingState,
    /// Cell IDs providing training data.
    pub data_sources: Vec<CellId>,
    /// Checkpoint intervals (epochs).
    pub checkpoint_every: u32,
}

impl TrainingCell {
    pub fn new(model: impl Into<String>, epochs: u32) -> Self {
        Self {
            model_name: model.into(),
            state: CellState::Idle,
            training: TrainingState { total_epochs: epochs, ..Default::default() },
            data_sources: vec![],
            checkpoint_every: 10,
        }
    }

    pub fn with_data_source(mut self, id: CellId) -> Self {
        self.data_sources.push(id);
        self
    }

    /// Simulate one training step.
    pub fn step(&mut self) -> CellValue {
        if self.training.current_epoch >= self.training.total_epochs {
            self.state = CellState::Ready;
            return CellValue::Text("Training complete".into());
        }
        self.training.current_epoch += 1;
        // Simulated loss decay
        self.training.loss = 1.0 / (self.training.current_epoch as f64).sqrt();
        if self.training.loss < self.training.best_loss {
            self.training.best_loss = self.training.loss;
        }
        self.state = CellState::Running;
        CellValue::Number(self.training.loss)
    }

    /// Current progress as a fraction (0.0–1.0).
    pub fn progress(&self) -> f64 {
        if self.training.total_epochs == 0 { return 0.0; }
        self.training.current_epoch as f64 / self.training.total_epochs as f64
    }

    /// Is training done?
    pub fn is_done(&self) -> bool {
        self.training.current_epoch >= self.training.total_epochs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_training_step() {
        let mut t = TrainingCell::new("test-model", 10);
        let result = t.step();
        assert_eq!(t.training.current_epoch, 1);
        assert_eq!(t.state, CellState::Running);
        assert!(matches!(result, CellValue::Number(_)));
    }

    #[test]
    fn test_training_complete() {
        let mut t = TrainingCell::new("test-model", 2);
        t.step();
        t.step();
        let result = t.step();
        assert!(t.is_done());
        assert_eq!(t.state, CellState::Ready);
    }

    #[test]
    fn test_training_progress() {
        let mut t = TrainingCell::new("test-model", 100);
        t.step();
        t.step();
        assert!(t.progress() > 0.0);
        assert!(t.progress() < 0.1);
    }

    #[test]
    fn test_best_loss_tracking() {
        let mut t = TrainingCell::new("test-model", 100);
        t.step();
        let initial_best = t.training.best_loss;
        t.step();
        assert!(t.training.best_loss <= initial_best);
    }
}
