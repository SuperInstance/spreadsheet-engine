//! A2A cell protocol — cells discover and communicate with each other.
//!
//! In the living spreadsheet, A2A cells are first-class citizens. They can
//! announce their presence, query other cells, and exchange messages — all
//! through the grid's [`A2ABus`].
//!
//! # Protocol Overview
//!
//! The A2A protocol follows a simple announce → query → update cycle:
//! 1. Cells **announce** their capabilities on the bus
//! 2. Other cells **query** for specific capabilities
//! 3. Cells **send** messages (Train, Simulate, Midi commands)
//! 4. Recipients **drain** their inbox and process messages
//!
//! # Example
//!
//! ```
//! use spreadsheet_engine::a2a::{A2ABus, A2AMessage, A2AMessageKind, A2ACell};
//! use spreadsheet_engine::cell::{CellId, CellValue};
//!
//! let mut bus = A2ABus::new();
//! let a = CellId::new(0, 0);
//! let b = CellId::new(0, 1);
//!
//! bus.announce(a, vec!["ml".into()], 0);
//! bus.announce(b, vec!["midi".into()], 0);
//!
//! let ml_agents = bus.find_by_capability("ml");
//! assert!(ml_agents.contains(&a));
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::cell::{CellId, CellState, CellValue};

// ── A2A Message ──────────────────────────────────────────────────────

/// A message between cells in the A2A protocol.
///
/// Messages carry a payload (any `CellValue`), a tick timestamp,
/// and a kind that determines how the recipient should handle it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    /// The cell that sent this message.
    pub from: CellId,
    /// The destination cell.
    pub to: CellId,
    /// What kind of message this is (determines handling).
    pub kind: A2AMessageKind,
    /// The message payload — any cell value.
    pub payload: CellValue,
    /// The tick at which this message was sent.
    pub tick: u64,
}

/// Types of A2A messages.
///
/// Each variant corresponds to a specific phase of inter-cell communication:
/// - Discovery: `Announce`, `Query`
/// - State sync: `Update`
/// - Commands: `Train`, `Simulate`, `Midi`
/// - Lifecycle: `Complete`, `Error`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum A2AMessageKind {
    /// "I exist and have these capabilities."
    Announce,
    /// "What can you do? / What's your value?"
    Query,
    /// "Here's my current state."
    Update,
    /// "Start a training run with this data."
    Train,
    /// "Advance your simulation by one tick."
    Simulate,
    /// "Generate a MIDI event from your state."
    Midi,
    /// "I'm done with this task."
    Complete,
    /// "Something went wrong."
    Error,
}

// ── A2A Bus ──────────────────────────────────────────────────────────

/// The message bus that routes A2A messages between cells.
///
/// The bus maintains:
/// - An **inbox** per cell (pending messages)
/// - A **registry** of announced cells and their capabilities
///
/// Messages are queued on `send()` and drained on `drain()`. There's no
/// delivery callback — cells poll their inbox when ready.
#[derive(Debug, Clone, Default)]
pub struct A2ABus {
    /// Pending messages per destination cell.
    inbox: HashMap<CellId, Vec<A2AMessage>>,
    /// Registry of announced cells and their capabilities.
    registry: HashMap<CellId, Vec<String>>,
}

impl A2ABus {
    /// Create a new empty A2A bus.
    pub fn new() -> Self {
        Self::default()
    }

    /// Send a message. It will be queued in the recipient's inbox.
    pub fn send(&mut self, msg: A2AMessage) {
        self.inbox.entry(msg.to).or_default().push(msg);
    }

    /// Broadcast an announcement to all known cells.
    ///
    /// Registers the sending cell's capabilities and sends an `Announce`
    /// message to every other known cell.
    pub fn announce(&mut self, from: CellId, capabilities: Vec<String>, tick: u64) {
        self.registry.insert(from, capabilities.clone());
        let targets: Vec<CellId> = self.registry.keys().copied()
            .filter(|id| *id != from)
            .collect();
        for to in targets {
            self.send(A2AMessage {
                from, to,
                kind: A2AMessageKind::Announce,
                payload: CellValue::Text(format!("caps:{}", capabilities.join(","))),
                tick,
            });
        }
    }

    /// Drain all pending messages for a cell.
    ///
    /// Returns all messages and removes them from the inbox.
    pub fn drain(&mut self, id: &CellId) -> Vec<A2AMessage> {
        self.inbox.remove(id).unwrap_or_default()
    }

    /// Get the number of pending messages for a cell.
    pub fn pending(&self, id: &CellId) -> usize {
        self.inbox.get(id).map(|v| v.len()).unwrap_or(0)
    }

    /// Find cells with a specific capability.
    pub fn find_by_capability(&self, cap: &str) -> Vec<CellId> {
        self.registry.iter()
            .filter(|(_, caps)| caps.iter().any(|c| c == cap))
            .map(|(&id, _)| id)
            .collect()
    }

    /// All announced cells.
    pub fn announced(&self) -> Vec<CellId> {
        self.registry.keys().copied().collect()
    }

    /// Total messages queued across all inboxes.
    pub fn total_queued(&self) -> usize {
        self.inbox.values().map(|v| v.len()).sum()
    }
}

// ── A2A Cell ─────────────────────────────────────────────────────────

/// An A2A endpoint cell — represents an agent that communicates via the bus.
///
/// A2A cells respond to `Query` messages with their last value,
/// and update their state when receiving `Update` messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2ACell {
    /// Unique identifier for this agent.
    pub agent_id: String,
    /// List of capability strings this agent provides.
    pub capabilities: Vec<String>,
    /// Current lifecycle state.
    pub state: CellState,
    /// The last computed value of this cell.
    pub last_value: CellValue,
}

impl A2ACell {
    /// Create a new A2A cell with the given agent ID and capabilities.
    pub fn new(id: impl Into<String>, capabilities: Vec<String>) -> Self {
        Self {
            agent_id: id.into(),
            capabilities,
            state: CellState::Idle,
            last_value: CellValue::Empty,
        }
    }

    /// Process incoming messages and return responses.
    ///
    /// - `Query` messages generate `Update` responses with the cell's current value.
    /// - `Update` messages update the cell's last value.
    /// - Other message types are ignored (for now).
    pub fn process_messages(&mut self, messages: &[A2AMessage]) -> Vec<A2AMessage> {
        let mut responses = Vec::new();
        for msg in messages {
            match msg.kind {
                A2AMessageKind::Query => {
                    responses.push(A2AMessage {
                        from: msg.to,
                        to: msg.from,
                        kind: A2AMessageKind::Update,
                        payload: self.last_value.clone(),
                        tick: msg.tick,
                    });
                }
                A2AMessageKind::Update => {
                    self.last_value = msg.payload.clone();
                }
                _ => {}
            }
        }
        responses
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a2a_send_and_drain() {
        let mut bus = A2ABus::new();
        let a = CellId::new(0, 0);
        let b = CellId::new(0, 1);
        bus.send(A2AMessage { from: a, to: b, kind: A2AMessageKind::Query, payload: CellValue::Empty, tick: 0 });
        assert_eq!(bus.pending(&b), 1);
        let msgs = bus.drain(&b);
        assert_eq!(msgs.len(), 1);
        assert_eq!(bus.pending(&b), 0);
    }

    #[test]
    fn test_announce() {
        let mut bus = A2ABus::new();
        let a = CellId::new(0, 0);
        let b = CellId::new(0, 1);
        bus.announce(a, vec!["rust".into()], 0);
        bus.announce(b, vec!["python".into()], 0);
        assert_eq!(bus.announced().len(), 2);
    }

    #[test]
    fn test_find_by_capability() {
        let mut bus = A2ABus::new();
        let a = CellId::new(0, 0);
        bus.announce(a, vec!["rust".into(), "ml".into()], 0);
        let found = bus.find_by_capability("ml");
        assert!(found.contains(&a));
        assert!(bus.find_by_capability("python").is_empty());
    }

    #[test]
    fn test_a2a_cell_process() {
        let mut cell = A2ACell::new("agent-1", vec!["rust".into()]);
        let msgs = vec![A2AMessage {
            from: CellId::new(1, 0),
            to: CellId::new(0, 0),
            kind: A2AMessageKind::Query,
            payload: CellValue::Empty,
            tick: 0,
        }];
        let responses = cell.process_messages(&msgs);
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0].kind, A2AMessageKind::Update);
    }

    #[test]
    fn test_total_queued() {
        let mut bus = A2ABus::new();
        let a = CellId::new(0, 0);
        let b = CellId::new(0, 1);
        bus.send(A2AMessage { from: a, to: b, kind: A2AMessageKind::Query, payload: CellValue::Empty, tick: 0 });
        bus.send(A2AMessage { from: b, to: a, kind: A2AMessageKind::Update, payload: CellValue::Number(42.0), tick: 0 });
        assert_eq!(bus.total_queued(), 2);
    }
}
