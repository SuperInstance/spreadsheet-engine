//! Cell types and evaluation model for the living spreadsheet.
//!
//! Every cell implements the [`CellEval`] trait for async evaluation within
//! the engine's tick loop. Cells can depend on other cells, forming a directed
//! acyclic evaluation graph.

use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};

// ── Cell Address ─────────────────────────────────────────────────────

/// Row/column address of a cell in the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellId {
    pub row: u32,
    pub col: u32,
}

impl CellId {
    pub fn new(row: u32, col: u32) -> Self {
        Self { row, col }
    }

    /// Excel-style label: A1, B2, AA7, etc.
    pub fn label(&self) -> String {
        let mut col_name = String::new();
        let mut c = self.col;
        loop {
            col_name.insert(0, (b'A' + (c % 26) as u8) as char);
            c = if c < 26 { break } else { c / 26 - 1 };
        }
        format!("{}{}", col_name, self.row + 1)
    }
}

impl fmt::Display for CellId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ── Cell Values ──────────────────────────────────────────────────────

/// A computed value from a cell evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellValue {
    Number(f64),
    Text(String),
    Bool(bool),
    /// A ternary value from the fleet's balanced ternary system.
    Ternary(i8), // -1, 0, +1
    /// A vector of values (e.g., agent state, population).
    Vector(Vec<f64>),
    /// No value yet (unevaluated).
    Empty,
    /// An error from evaluation.
    Error(String),
}

impl CellValue {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            CellValue::Number(n) => Some(*n),
            CellValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, CellValue::Empty)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, CellValue::Error(_))
    }
}

impl From<i64> for CellValue {
    fn from(n: i64) -> Self { CellValue::Number(n as f64) }
}

impl From<f64> for CellValue {
    fn from(n: f64) -> Self { CellValue::Number(n) }
}

impl From<bool> for CellValue {
    fn from(b: bool) -> Self { CellValue::Bool(b) }
}

impl From<String> for CellValue {
    fn from(s: String) -> Self { CellValue::Text(s) }
}

impl From<&str> for CellValue {
    fn from(s: &str) -> Self { CellValue::Text(s.into()) }
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CellValue::Number(n) => write!(f, "{}", n),
            CellValue::Text(s) => write!(f, "{}", s),
            CellValue::Bool(b) => write!(f, "{}", b),
            CellValue::Ternary(t) => write!(f, "{:+}", t),
            CellValue::Vector(v) => write!(f, "[{}]", v.iter().map(|x| format!("{:.2}", x)).collect::<Vec<_>>().join(",")),
            CellValue::Empty => write!(f, "∅"),
            CellValue::Error(e) => write!(f, "ERR: {}", e),
        }
    }
}

// ── Cell Result ──────────────────────────────────────────────────────

/// Result of evaluating a cell.
#[derive(Debug, Clone)]
pub struct CellResult {
    pub value: CellValue,
    /// How long evaluation took.
    pub eval_time: Duration,
    /// Whether conservation constraints were satisfied.
    pub conservation_ok: bool,
}

impl CellResult {
    pub fn ok(value: CellValue) -> Self {
        Self { value, eval_time: Duration::ZERO, conservation_ok: true }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self { value: CellValue::Error(msg.into()), eval_time: Duration::ZERO, conservation_ok: false }
    }

    pub fn with_time(mut self, d: Duration) -> Self {
        self.eval_time = d;
        self
    }
}

// ── Eval Context ─────────────────────────────────────────────────────

/// Context passed to cells during evaluation — provides access to other cells.
#[derive(Debug, Clone)]
pub struct EvalContext {
    /// Values of cells this cell depends on (resolved before this cell).
    pub dependencies: HashMap<CellId, CellValue>,
    /// Current tick number (monotonically increasing).
    pub tick: u64,
    /// Grid-wide conservation budget.
    pub total_budget: f64,
}

// ── Cell State ───────────────────────────────────────────────────────

/// Lifecycle state of a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellState {
    /// Just created, not yet evaluated.
    Idle,
    /// Currently being evaluated.
    Evaluating,
    /// Successfully evaluated.
    Ready,
    /// Evaluation failed.
    Error,
    /// Actively training or simulating.
    Running,
    /// Paused by user or dependency.
    Paused,
}

// ── Concrete Cell Types ──────────────────────────────────────────────

/// A plain value cell — numbers, text, booleans.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueCell {
    pub value: CellValue,
}

impl ValueCell {
    pub fn from(v: impl Into<CellValue>) -> Self {
        Self { value: v.into() }
    }

    pub fn number(n: f64) -> Self {
        Self { value: CellValue::Number(n) }
    }

    pub fn text(s: impl Into<String>) -> Self {
        Self { value: CellValue::Text(s.into()) }
    }
}

impl From<i64> for ValueCell {
    fn from(n: i64) -> Self { Self { value: CellValue::Number(n as f64) } }
}

impl From<f64> for ValueCell {
    fn from(n: f64) -> Self { Self { value: CellValue::Number(n) } }
}

/// An AI agent cell with capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCell {
    pub agent_id: String,
    /// Capability scores: name → confidence (0.0–1.0).
    pub capabilities: HashMap<String, f64>,
    /// Gamma — compute spend (conservation).
    pub gamma: f64,
    /// Eta — memory usage (conservation).
    pub eta: f64,
    /// Individual budget.
    pub budget: f64,
    pub state: CellState,
}

impl AgentCell {
    pub fn new(id: impl Into<String>, budget: f64) -> Self {
        Self {
            agent_id: id.into(),
            capabilities: HashMap::new(),
            gamma: 0.0,
            eta: 0.0,
            budget,
            state: CellState::Idle,
        }
    }

    pub fn with_capability(mut self, name: &str, confidence: f64) -> Self {
        self.capabilities.insert(name.into(), confidence);
        self
    }

    /// Conservation error: |γ + η - budget|.
    pub fn conservation_error(&self) -> f64 {
        (self.gamma + self.eta - self.budget).abs()
    }

    /// Is this agent within its budget?
    pub fn is_healthy(&self, tolerance: f64) -> bool {
        self.conservation_error() <= tolerance
    }
}

// ── Cell Enum ────────────────────────────────────────────────────────

/// A cell in the living spreadsheet. Each variant represents a different
/// kind of computational unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Cell {
    Value(ValueCell),
    Agent(AgentCell),
    Training(crate::training::TrainingCell),
    Simulation(crate::simulation::SimulationCell),
    A2A(crate::a2a::A2ACell),
    Midi(crate::midi::MidiCell),
    Formula(crate::formula::FormulaCell),
}

impl Cell {
    /// Current state of this cell.
    pub fn state(&self) -> CellState {
        match self {
            Cell::Value(_) => CellState::Ready,
            Cell::Agent(a) => a.state,
            Cell::Training(t) => t.state,
            Cell::Simulation(s) => s.state,
            Cell::A2A(a) => a.state,
            Cell::Midi(m) => m.state,
            Cell::Formula(f) => f.state,
        }
    }

    /// Human-readable type name.
    pub fn type_name(&self) -> &'static str {
        match self {
            Cell::Value(_) => "Value",
            Cell::Agent(_) => "Agent",
            Cell::Training(_) => "Training",
            Cell::Simulation(_) => "Simulation",
            Cell::A2A(_) => "A2A",
            Cell::Midi(_) => "MIDI",
            Cell::Formula(_) => "Formula",
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_id_label() {
        assert_eq!(CellId::new(0, 0).label(), "A1");
        assert_eq!(CellId::new(0, 1).label(), "B1");
        assert_eq!(CellId::new(2, 0).label(), "A3");
    }

    #[test]
    fn test_cell_id_display() {
        assert_eq!(format!("{}", CellId::new(0, 0)), "A1");
    }

    #[test]
    fn test_value_cell_number() {
        let v = ValueCell::from(42);
        assert_eq!(v.value, CellValue::Number(42.0));
    }

    #[test]
    fn test_value_cell_text() {
        let v = ValueCell::text("hello");
        assert_eq!(v.value, CellValue::Text("hello".into()));
    }

    #[test]
    fn test_cell_value_as_f64() {
        assert_eq!(CellValue::Number(3.14).as_f64(), Some(3.14));
        assert_eq!(CellValue::Bool(true).as_f64(), Some(1.0));
        assert_eq!(CellValue::Text("x".into()).as_f64(), None);
    }

    #[test]
    fn test_cell_value_display() {
        assert_eq!(format!("{}", CellValue::Number(42.0)), "42");
        assert_eq!(format!("{}", CellValue::Empty), "∅");
        assert_eq!(format!("{}", CellValue::Ternary(-1)), "-1");
    }

    #[test]
    fn test_agent_cell() {
        let agent = AgentCell::new("alice", 1.0)
            .with_capability("rust", 0.9)
            .with_capability("python", 0.7);
        assert_eq!(agent.capabilities.len(), 2);
        // Fresh agent: gamma=0, eta=0, budget=1.0 → error=1.0
        assert!(!agent.is_healthy(0.01)); // Not healthy until gamma+eta are set
    }

    #[test]
    fn test_agent_conservation_error() {
        let mut agent = AgentCell::new("bob", 1.0);
        agent.gamma = 0.6;
        agent.eta = 0.3;
        // γ+η = 0.9, budget = 1.0, error = |0.9 - 1.0| = 0.1
        assert!((agent.conservation_error() - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_agent_healthy_when_balanced() {
        let mut agent = AgentCell::new("carol", 1.0);
        agent.gamma = 0.6;
        agent.eta = 0.4;
        // γ+η = 1.0 = budget → error = 0
        assert!(agent.is_healthy(0.01));
    }

    #[test]
    fn test_cell_type_name() {
        assert_eq!(Cell::Value(ValueCell::from(1)).type_name(), "Value");
        assert_eq!(Cell::Agent(AgentCell::new("x", 1.0)).type_name(), "Agent");
    }

    #[test]
    fn test_cell_result_ok() {
        let r = CellResult::ok(CellValue::Number(42.0));
        assert!(r.conservation_ok);
        assert!(r.value.as_f64() == Some(42.0));
    }

    #[test]
    fn test_cell_result_err() {
        let r = CellResult::err("boom");
        assert!(!r.conservation_ok);
        assert!(r.value.is_error());
    }
}
