# spreadsheet-engine

[![crates.io](https://img.shields.io/crates/v/spreadsheet-engine.svg)](https://crates.io/crates/spreadsheet-engine)
[![license: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![clippy](https://img.shields.io/badge/clippy-clean-green.svg)]()

**Core engine for living AI spreadsheets — every cell can be an agent, a training job, a simulation, or a MIDI generator.**

## The Problem

Spreadsheets are the most successful programming model in history. A billion people use them. But they're static — cells hold numbers and formulas, not *intelligences*. What if a cell could hold an agent that evolves, a training job that converges, a simulation that runs, or a MIDI generator that composes?

The challenge is making this composable: agents need to communicate, evolution needs to conserve, and the whole thing needs to evaluate in dependency order like a real spreadsheet.

## The Insight

A cell is just a typed value with a tick function. The types form an algebra:

```
Cell = Value | Agent | Training | Simulation | A2A | MIDI | Formula
```

Each type has its own evaluation semantics:
- **Value**: static data (number, text, ternary {-1,0,+1}, vector)
- **Agent**: ternary strategy {-1,0,+1}^N with fitness scoring
- **Training**: convergence curve (1/sqrt(epoch) decay)
- **Simulation**: damped oscillation (1D dynamical system)
- **A2A**: inter-cell message bus (announce/discover/query)
- **MIDI**: ternary→pitch-class sonification
- **Formula**: =ENTROPY, =PARETO, =SPECIES, =EVOLVE, =CONSERVATION

The engine evaluates cells in topological order (Kahn's algorithm on the dependency DAG) with cycle detection. Each tick advances all cells one step.

A **conservation monitor** tracks the fleet health: γ (gamma = conservation score) + η (eta = entropy) must stay within budget across agent cells. If it drifts, the monitor flags it.

## How It Works

### Cell Types

```rust
use spreadsheet_engine::{Cell, CellValue, ValueCell, AgentCell};

// A static value
let val = Cell::Value(ValueCell::from(42));

// A ternary agent
let agent = Cell::Agent(AgentCell::new(
    vec![-1, 0, 1, 1],  // strategy weights
    0.85,                // gamma (conservation)
    0.12,                // eta (entropy)
    1.0,                 // budget
));

// A formula cell
let formula = Cell::Formula(FormulaCell::parse("=ENTROPY(A1:A10)").unwrap());
```

### Grid and Evaluation

```rust
use spreadsheet_engine::{Grid, Engine};

let mut engine = Engine::new();
engine.grid_mut().insert("A1".parse()?, Cell::Value(ValueCell::from(3.14)));
engine.grid_mut().insert("A2".parse()?, agent);
engine.tick()?;  // Evaluate all cells in dependency order
```

### Conservation Monitor

```rust
// The conservation law: γ + η ≤ budget for each agent cell
// γ = conservation score (how well the agent preserves fleet invariants)
// η = entropy (Shannon entropy of the agent's strategy distribution)
// If γ + η drifts above budget, the monitor flags ConservationStatus::Degrading
```

### Formula System

| Formula | What it computes |
|---------|-----------------|
| `=ENTROPY(range)` | Shannon entropy of ternary values |
| `=PARETO(range)` | Pareto-optimal agents (non-dominated) |
| `=SPECIES(range, k)` | K-means clustering of strategies |
| `=EVOLVE(range, gens)` | Genetic optimizer: top-50% survival + mutation |
| `=CONSERVATION(range)` | Fleet conservation status (γ + η vs budget) |

### MIDI Sonification

Agent strategies map to pitch classes via the ternary→MIDI bridge:
```
{-1, 0, +1} → {avoid, neutral, choose}
↓
pitch class mapping (C=0, C#=1, ..., B=11)
↓
MIDI note-on/note-off events
```

## Module Map

```
src/
├── cell.rs          Cell enum, CellValue, 7 cell types
├── engine.rs        Engine: tick, dependency resolution, evaluation
├── grid.rs          Grid: 2D cell storage, insert/remove/get
├── formula.rs       Formula parser, ENTROPY/PARETO/SPECIES/EVOLVE/CONSERVATION
├── conservation.rs  Conservation monitor, fleet health tracking
├── midi.rs          Ternary → MIDI pitch class sonification
├── simulation.rs    Simulation cell: damped oscillation
├── training.rs      Training cell: convergence curve
├── a2a.rs           Inter-cell message bus (announce/discover/query)
└── error.rs         Error types
```

## Design Decisions

**Why ternary agents, not arbitrary floats?** Ternary {-1, 0, +1} strategies create tiny, enumerable search spaces. For N=4 weights, there are exactly 81 strategies — you can evaluate all of them in microseconds. This makes `=EXHAUSTIVE()` (in the companion `si-superinstance` pip package) trivially fast. The spreadsheet cell is the container; the ternary strategy is the intelligence.

**Why conservation monitoring?** Without constraints, agents diverge. The γ + η budget is a thermodynamic constraint: agents must balance conservation (stability) with entropy (exploration). This is borrowed from the fleet conservation law discovered across 155+ crates.

**Why topological evaluation?** Real spreadsheets evaluate formulas in dependency order. We do the same but with typed cells — an A2A cell can depend on an Agent cell's output, a MIDI cell can depend on a Formula cell's result. Kahn's algorithm with cycle detection prevents infinite loops.

**Why not async (yet)?** The engine is synchronous for simplicity. Cell evaluation is fast enough that async overhead isn't justified. The `tokio` dependency was removed in v0.1.1 after audit found it unused. If async evaluation becomes necessary (e.g., for real A2A networking), it can be added as a feature flag.

**Training and Simulation are placeholders.** The current implementations prove the tick mechanism works (`step()` returns a decay curve or oscillation), but they're not real ML or physics. The design intent is for these to become pluggable traits — you'd implement your own `TrainingAlgorithm` or `SimulationModel` and plug it into the cell.

## Related Crates

- **[si-superinstance](https://pypi.org/project/si-superinstance/)** — Python API for exhaustive ternary search (`pip install si-superinstance`)
- **[spreadsheet-plr-bridge](https://crates.io/crates/spreadsheet-plr-bridge)** — PLR group voice leading as spreadsheet formulas
- **[superinstance-spreadsheet](https://github.com/SuperInstance/superinstance-spreadsheet)** — Browser demo with =EXHAUSTIVE(), =EVOLVE(), charts

## Status

**v0.1.0 — Prototype.** The architecture is sound (cell-type algebra + DAG evaluation + conservation monitoring), but Training and Simulation cells are placeholders. The formula system, agent model, and conservation monitor are real and tested.

**Audit results** (2026-06-08): Clippy clean, 67 unit + 1 doc test passing. Test quality rated B+ by independent audit — formula and grid tests are substantive; cell construction tests are weaker.

## License

MIT
