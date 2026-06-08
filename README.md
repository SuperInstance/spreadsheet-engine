# spreadsheet-engine

**Core engine for living AI spreadsheets** — every cell can be an agent, a training job, a simulation, or a MIDI generator.

[![crates.io](https://img.shields.io/crates/v/spreadsheet-engine.svg)](https://crates.io/crates/spreadsheet-engine)
[![docs.rs](https://docs.rs/spreadsheet-engine/badge.svg)](https://docs.rs/spreadsheet-engine)

## The Idea

Spreadsheets are the most successful programming model ever invented. What if every cell wasn't just a number or formula, but a living computational unit?

```
┌─────────┬──────────┬───────────┬──────────┐
│ Value   │ Agent    │ Training  │ MIDI     │
│ 42      │ 🤖 alice │ 📉 model  │ 🎵 C4    │
├─────────┼──────────┼───────────┼──────────┤
│ Formula │ A2A      │ Simulation│ Formula  │
│ =EVOLVE │ 🔗 disc  │ ⚡ tick   │ =PARETO  │
└─────────┴──────────┴───────────┴──────────┘
```

## Quick Start

```rust
use spreadsheet_engine::{Grid, Engine, Cell, CellId, ValueCell, AgentCell};
use spreadsheet_engine::{FormulaCell, FormulaOp, SimulationCell};

// Create a grid
let mut grid = Grid::with_budget(10.0, 0.01);

// Add cells
grid.insert(CellId::new(0, 0), Cell::Value(ValueCell::from(42)));
grid.insert(CellId::new(0, 1), Cell::Agent(
    AgentCell::new("alice", 1.0).with_capability("rust", 0.9)
));

// Add an evolutionary formula
grid.insert(CellId::new(1, 0), Cell::Formula(
    FormulaCell::new(FormulaOp::Evolve {
        generations: 50, population_size: 20, mutation_rate: 0.3
    }, vec![CellId::new(0, 0), CellId::new(0, 1)])
));

// Run the engine
let mut engine = Engine::new(grid);
engine.tick().unwrap();

println!("Health: {:.2}%", engine.health() * 100.0);
```

## Cell Types

| Type | Purpose | Example |
|------|---------|---------|
| `Value` | Plain data (numbers, text, booleans, ternary) | `42`, `"hello"`, `{-1, 0, +1}` |
| `Agent` | AI agent with capabilities and conservation budget | LLM cell, classifier |
| `Training` | Active ML training job with epochs and loss tracking | Fine-tuning loop |
| `Simulation` | Tick-based simulation synchronized to fleet pulse | Physics sim, agent dynamics |
| `A2A` | Agent-to-agent endpoint with discovery | Inter-cell communication |
| `Midi` | MIDI event generator for sonification | Hear your spreadsheet |
| `Formula` | Evolutionary formula operations | `EVOLVE`, `PARETO`, `SPECIES` |

## Evolutionary Formulas

This is what makes it alive. Standard spreadsheets have `SUM` and `AVERAGE`. We have:

- **`EVOLVE`** — Genetic optimization over cell values. Mutate, select fittest, repeat.
- **`SPECIES`** — Cluster cells by similarity (k-means over value vectors).
- **`PARETO`** — Find Pareto-optimal cells (multi-objective optimization).
- **`ENTROPY`** — Shannon entropy of a range (measure diversity).
- **`CONSERVE`** — Track conservation law across cells (γ + η = budget).

Plus standard math: `ADD`, `SUB`, `MUL`, `DIV`, `SUM`, `AVERAGE`, `MAX`, `MIN`.

## Conservation Monitoring

Every agent cell tracks `γ` (compute spend) and `η` (memory usage) against a `budget`. The grid enforces:

```
γ + η ≤ budget  (for each agent)
Σ(γ) + Σ(η) ≤ total_budget  (for the grid)
```

Violations show up as conservation errors and propagate to MIDI cells as dissonance.

## A2A Protocol

Cells discover each other through the A2A bus:

```rust
// Cell announces its capabilities
bus.announce(cell_id, vec!["rust".into(), "ml".into()], tick);

// Find cells with a capability
let rust_cells = bus.find_by_capability("rust");

// Send a message
bus.send(A2AMessage {
    from: cell_a, to: cell_b,
    kind: A2AMessageKind::Query,
    payload: CellValue::Empty,
    tick,
});
```

## MIDI Sonification

Map cell values to sound. Conservation violations become audible:

```rust
let mut midi = MidiCell::new("health-sound", 0, 60); // Channel 0, C4
let events = midi.sonify(&CellValue::Number(0.0)); // Root = healthy
let events = midi.sonify(&CellValue::Ternary(-1)); // Minor third = degraded
```

## Architecture

```
Grid (sparse HashMap<CellId, Cell>)
  │
  ├── Engine (tick loop, dependency resolution)
  │     ├── A2ABus (inter-cell messaging)
  │     ├── ConservationMonitor (budget tracking)
  │     └── EvalContext (dependency values)
  │
  └── Cells: Value | Agent | Training | Simulation | A2A | Midi | Formula
```

## Related Crates

- [`cmidi-core`](https://crates.io/crates/cmidi-core) — Conversational MIDI protocol
- [`cmidi-conservation`](https://crates.io/crates/cmidi-conservation) — Conservation law → harmonic tension
- [`capability-spec`](https://crates.io/crates/capability-spec) — Agent capability specifications

## License

MIT
