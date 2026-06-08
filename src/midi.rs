//! MIDI cell — bridges grid state to cmidi-core for sonification.
//!
//! A MIDI cell converts its state into CMIDI events, allowing the grid
//! to be heard as music. Conservation violations become dissonance.

use serde::{Deserialize, Serialize};

use crate::cell::{CellState, CellValue};

/// A cell that generates MIDI events from its state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiCell {
    pub name: String,
    pub state: CellState,
    pub channel: u8,
    /// Base pitch (MIDI note number, 0-127).
    pub base_note: u8,
    /// Current velocity (1-127).
    pub velocity: u8,
    /// Last generated MIDI event bytes.
    pub last_event: Option<[u8; 3]>,
}

impl MidiCell {
    pub fn new(name: impl Into<String>, channel: u8, base_note: u8) -> Self {
        Self {
            name: name.into(),
            state: CellState::Idle,
            channel: channel.min(15),
            base_note: base_note.min(127),
            velocity: 100,
            last_event: None,
        }
    }

    /// Generate a MIDI note-on event from a cell value.
    /// Numbers map to pitch offset, vectors map to chords.
    pub fn sonify(&mut self, value: &CellValue) -> Vec<[u8; 3]> {
        let events = match value {
            CellValue::Number(n) => {
                // Map number to pitch offset from base_note
                let offset = (*n as i16).clamp(-12, 12) as i8;
                let note = (self.base_note as i16 + offset as i16).clamp(0, 127) as u8;
                let event = [0x90 | self.channel, note, self.velocity];
                self.last_event = Some(event);
                vec![event]
            }
            CellValue::Ternary(t) => {
                // Ternary → three notes: below/root/above
                let note = match t {
                    -1 => self.base_note.saturating_sub(2), // minor third below
                    0 => self.base_note,                     // root
                    1 => self.base_note.saturating_add(4),   // major third above
                    _ => self.base_note,
                };
                let event = [0x90 | self.channel, note.min(127), self.velocity];
                self.last_event = Some(event);
                vec![event]
            }
            CellValue::Vector(v) => {
                // Each element → one note in a chord
                v.iter().take(6).map(|&n| {
                    let offset = (n * 12.0).round() as i8;
                    let note = (self.base_note as i16 + offset as i16).clamp(0, 127) as u8;
                    [0x90 | self.channel, note, self.velocity]
                }).collect()
            }
            _ => vec![],
        };
        self.state = CellState::Ready;
        events
    }

    /// Generate a note-off event.
    pub fn note_off(&self) -> [u8; 3] {
        let note = self.last_event.map(|e| e[1]).unwrap_or(self.base_note);
        [0x80 | self.channel, note, 0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_sonify_number() {
        let mut m = MidiCell::new("test", 0, 60); // C4
        let events = m.sonify(&CellValue::Number(0.0));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0][1], 60); // C4
    }

    #[test]
    fn test_midi_sonify_ternary() {
        let mut m = MidiCell::new("test", 0, 60);
        let events = m.sonify(&CellValue::Ternary(1));
        assert_eq!(events[0][1], 64); // E4 (major third above C4)
    }

    #[test]
    fn test_midi_sonify_vector() {
        let mut m = MidiCell::new("test", 0, 60);
        let events = m.sonify(&CellValue::Vector(vec![0.0, 1.0, -1.0]));
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_midi_note_off() {
        let mut m = MidiCell::new("test", 0, 60);
        m.sonify(&CellValue::Number(2.0));
        let off = m.note_off();
        assert_eq!(off[0], 0x80);
        assert_eq!(off[2], 0);
    }

    #[test]
    fn test_midi_channel_clamp() {
        let m = MidiCell::new("test", 20, 60); // channel > 15
        assert_eq!(m.channel, 15);
    }
}
