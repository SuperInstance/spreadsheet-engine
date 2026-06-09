# spreadsheet-engine

**An AI spreadsheet where every cell can be an agent, a training job, a simulation, an A2A endpoint, or a MIDI generator — and conservation laws keep the whole thing honest.**

## The Problem

Spreadsheets are the most successful programming model ever invented. But they're stuck in 1979: cells hold numbers and text, formulas are simple arithmetic, and there's no concept of *computation as a living process*. Meanwhile, AI agents train models, run simulations, talk to each other — but they live in ad-hoc scripts and notebooks, not in a unified grid where dependencies are explicit and conservation is enforced.

This crate asks: **what if every cell in a spreadsheet was a first-class computational agent with its own budget, lifecycle, and communication protocol?**

## The Key Insight

The spreadsheet grid is a **dependency graph with a conservation law**. Each agent cell has a compute budget governed by γ (compute spend) + η (memory usage) = budget. This is not arbitrary — it's the same insight from Noether's theorem: if your system has a symmetry (budget invariance), there's a conserved quantity. The `ConservationMonitor` tracks this across the entire fleet, computing a health score that tells you if your living spreadsheet is thermodynamically sound.

When γ + η ≠ budget, that's not just a bug — it's a *conservation violation*. The grid detects it, tracks trends (improving/degrading), and identifies which specific cells are leaking.

## Architecture

```
                    ┌─────────────────────────────────────┐
                    │            Engine                     │
                    │  (tick loop, dependency eval order)   │
                    └──────────┬──────────────────────────┘
                               │
                    ┌──────────▼──────────────────────────┐
                    │             Grid                      │
                    │  (sparse HashMap + dependency graph)  │
                    │  (topological sort, cycle detection)  │
                    └──────────┬──────────────────────────┘
                               │
        ┌──────────┬───────────┼───────────┬──────────────┐
        │          │           │           │              │
   ┌────▼───┐ ┌───▼────┐ ┌───▼────┐ ┌────▼─────┐ ┌─────▼─────┐
   │ Value  │ │ Agent  │ │Training│ │Simulation│ │   A2A     │
   │  Cell  │ │  Cell  │ │  Cell  │ │   Cell   │ │   Cell    │
   └────────┘ └────────┘ └────────┘ └──────────┘ └───────────┘
        │          │           │           │              │
        └──────────┴───────────┼───────────┴──────────────┘
                               │
                    ┌──────────▼──────────────────────────┐
                    │          Formula Cell                 │
                    │  (SUM, AVERAGE, EVOLVE, SPECIES,     │
                    │   PARETO, ENTROPY, CONSERVE)          │
                    └──────────────────────────────────────┘
                               │
                    ┌──────────▼──────────────────────────┐
                    │         MIDI Cell                     │
                    │  (sonify any cell value as MIDI)      │
                    └──────────────────────────────────────┘
                               │
                    ┌──────────▼──────────────────────────┐
                    │     Conservation Monitor              │
                    │  (fleet-wide γ + η = budget check)    │
                    └──────────────────────────────────────┘
                               │
                    ┌──────────▼──────────────────────────┐
                    │          A2A Bus                      │
                    │  (inter-cell message routing)         │
                    └──────────────────────────────────────┘
```

### Module Breakdown

| Module | What it does |
|--------|-------------|
| `cell` | 7 cell types, cell values, evaluation context, lifecycle states |
| `grid` | Sparse grid with dependency tracking and topological sort |
| `engine` | Tick loop that evaluates cells in dependency order |
| `formula` | Standard + evolutionary formulas (EVOLVE, SPECIES, PARETO, ENTROPY, CONSERVE) |
| `training` | ML training jobs as cells with epoch tracking |
| `simulation` | Simulations as cells with state vectors and tick counters |
| `a2a` | Agent-to-agent message bus with capability discovery |
| `midi` | MIDI generation from cell values |
| `conservation` | Fleet-wide conservation law monitoring |

### The 7 Cell Types

1. **Value** — Plain data (numbers, text, booleans, ternary, vectors)
2. **Agent** — An AI agent with capabilities, compute budget (γ), and memory budget (η)
3. **Training** — An ML training job with epochs, loss tracking, and checkpoints
4. **Simulation** — A simulation with a state vector, tick counter, and max steps
5. **A2A** — An agent-to-agent endpoint that discovers and communicates with other cells
6. **MIDI** — Sonifies cell values as MIDI events
7. **Formula** — Computes over a range of input cells (standard + evolutionary ops)

## Quick Start

```rust
use spreadsheet_engine::{
    Grid, Engine, Cell,
    cell::{CellId, ValueCell, AgentCell},
    formula::{FormulaCell, FormulaOp},
};

// Create a grid with a conservation budget
let mut grid = Grid::with_budget(2.0, 0.01);

// A1: simple value
let a1 = CellId::new(0, 0);
grid.insert(a1, Cell::Value(ValueCell::from(42)));

// B1: AI agent with capabilities and budget
let b1 = CellId::new(0, 1);
let agent = AgentCell::new("gpt-4", 1.0)
    .with_capability("rust", 0.9)
    .with_capability("python", 0.7);
grid.insert(b1, Cell::Agent(agent));

// C1: formula that sums A1 and B1's score
let c1 = CellId::new(0, 2);
let formula = FormulaCell::new(FormulaOp::Sum, vec![a1, b1]);
grid.insert(c1, Cell::Formula(formula));
grid.add_dependency(c1, a1);
grid.add_dependency(c1, b1);

// Run the engine
let mut engine = Engine::new(grid);
engine.run_ticks(10).unwrap();

// Check values
println!("A1 = {:?}", engine.value(&a1));  // Number(42.0)
println!("C1 = {:?}", engine.value(&c1));  // Sum of A1 + B1's agent score
println!("Grid health: {:.1}%", engine.health() * 100.0);
```

## Evolutionary Formulas

The formula cell supports operations beyond traditional spreadsheets:

### EVOLVE — Genetic Optimization
```rust
use spreadsheet_engine::formula::{FormulaCell, FormulaOp};

// Evolve values over 50 generations with a population of 30
let evolve = FormulaCell::new(
    FormulaOp::Evolve {
        generations: 50,
        population_size: 30,
        mutation_rate: 0.2,
    },
    vec![a1, b1, c1],
);
```

The `EVOLVE` operation treats input values as a seed population, applies random mutations, selects the fittest (highest total energy), and breeds the next generation. It returns the fittest individual as a `CellValue::Vector`.

### ENTROPY — Diversity Measurement
```rust
let entropy = FormulaCell::new(FormulaOp::Entropy, vec![a1, b1, c1, d1]);
```

Computes Shannon entropy H = −Σ pᵢ log₂(pᵢ) of the input range. All identical values → H = 0. Maximally diverse → H = log₂(n).

### SPECIES — Clustering
```rust
let species = FormulaCell::new(FormulaOp::Species { k: 3 }, vec![a1, b1, c1]);
```

Divides the input range into k equal bins and counts how many bins are occupied. This is a simplified k-means that reveals the "species" in your data.

### PARETO — Multi-Objective Optimization
```rust
let pareto = FormulaCell::new(FormulaOp::Pareto, vec![
    a1, b1, // point 1: (a1, b1)
    c1, d1, // point 2: (c1, d1)
    e1, f1, // point 3: (e1, f1)
]);
```

Treats consecutive pairs as (x, y) objectives and counts the Pareto-optimal (non-dominated) points.

### CONSERVE — Conservation Tracking
```rust
let conserve = FormulaCell::new(
    FormulaOp::Conserve,
    vec![gamma_cell, eta_cell, budget_cell],
);
```

Treats triplets as (γ, η, budget) and computes a health score: 1.0 when γ + η = budget, dropping toward 0.0 as the violation grows.

## Conservation Law Architecture

The conservation system is the mathematical backbone of the engine:

```
For each Agent cell:
  γ (compute spend) + η (memory usage) = budget (allocated resources)

Fleet-wide:
  Σᵢ γᵢ + Σᵢ ηᵢ = total_budget

ConservationMonitor tracks:
  - health = 1 − |Σγ + Ση − total_budget| / total_budget
  - violations = cells where |γ + η − budget| > tolerance
  - trend = improving | stable | degrading
```

This is not arbitrary — it's a direct application of Noether's theorem from the `noether-guard` crate. The symmetry is budget invariance; the conserved quantity is the budget itself.

## Inter-Cell Communication (A2A)

Cells can discover and communicate with each other through the A2A bus:

```rust
use spreadsheet_engine::a2a::{A2ABus, A2AMessage, A2AMessageKind, A2ACell};
use spreadsheet_engine::cell::{CellId, CellValue};

let mut bus = A2ABus::new();
let a = CellId::new(0, 0);
let b = CellId::new(0, 1);

// Announce capabilities
bus.announce(a, vec!["rust".into(), "ml".into()], 0);
bus.announce(b, vec!["python".into()], 0);

// Find agents with specific capabilities
let ml_agents = bus.find_by_capability("ml");
assert!(ml_agents.contains(&a));

// Send a message
bus.send(A2AMessage {
    from: a, to: b,
    kind: A2AMessageKind::Query,
    payload: CellValue::Empty,
    tick: 0,
});

// Drain inbox
let messages = bus.drain(&b);
```

## Training Cells

ML training jobs are first-class cells with epoch tracking and loss curves:

```rust
use spreadsheet_engine::training::TrainingCell;
use spreadsheet_engine::cell::CellValue;

let mut training = TrainingCell::new("resnet-50", 100); // 100 epochs
loop {
    let result = training.step();
    if training.is_done() { break; }
    if let CellValue::Number(loss) = result {
        println!("Epoch {}: loss = {:.4}", training.training.current_epoch, loss);
    }
}
println!("Best loss: {:.4}", training.training.best_loss);
```

Loss follows a simulated decay: L(t) = 1/√t.

## Simulation Cells

```rust
use spreadsheet_engine::simulation::SimulationCell;

let mut sim = SimulationCell::new("fluid-dynamics", 3)
    .with_state(vec![1.0, 0.0, -1.0]);
sim.sim.max_ticks = 100;

for _ in 0..10 {
    sim.step();
}
println!("Progress: {:.0}%", sim.progress() * 100.0);
```

## Performance

- **Grid operations**: O(1) cell lookup via HashMap
- **Dependency resolution**: O(V + E) topological sort (Kahn's algorithm)
- **Cycle detection**: O(V + E) DFS during dependency addition
- **Conservation check**: O(N) where N = number of agent cells
- **Formula evaluation**: O(n) for standard ops, O(g × p × n) for EVOLVE where g = generations, p = population size

The engine is designed for real-time use — a tick should complete in microseconds for typical grids.

## SuperInstance Ecosystem

`spreadsheet-engine` is the flagship crate of the SuperInstance project. It integrates with:

| Crate | Integration |
|-------|------------|
| `noether-guard` | Conservation law checking via Noether's theorem |
| `groovemesh-plr` | PLR voice-leading for MIDI cells |
| `lotka-beats` | Lotka-Volterra dynamics for simulation cells |
| `tropical-synth` | Tropical geometry for sound design in MIDI cells |

## API Overview

### Core Types

- `Grid` — Sparse cell grid with dependency tracking
- `Engine` — Tick-based evaluation loop
- `Cell` — Enum of 7 cell types
- `CellId` — (row, col) address with Excel-style labels
- `CellValue` — Number, Text, Bool, Ternary, Vector, Empty, Error
- `CellResult` — Evaluation result with timing and conservation status

### Cell Types

- `ValueCell` — Plain data
- `AgentCell` — AI agent with capabilities and budget
- `TrainingCell` — ML training job
- `SimulationCell` — Simulation with state vector
- `A2ACell` — Agent-to-agent endpoint
- `MidiCell` — MIDI event generator
- `FormulaCell` — Standard + evolutionary formula evaluation

### Infrastructure

- `A2ABus` — Inter-cell message routing
- `ConservationMonitor` — Fleet-wide budget tracking
- `EvalContext` — Dependencies and budget context for cell evaluation

## Comparison

| Feature | spreadsheet-engine | Traditional spreadsheets | Agent frameworks |
|---------|-------------------|------------------------|-----------------|
| Cell types | 7 (value, agent, training, sim, A2A, MIDI, formula) | 1 (value) | N/A |
| Dependencies | Explicit DAG with cycle detection | Implicit reference chains | N/A |
| Conservation | γ + η = budget enforcement | None | Resource limits |
| Evolutionary ops | EVOLVE, SPECIES, PARETO, ENTROPY, CONSERVE | SUM, AVERAGE, VLOOKUP | N/A |
| Inter-cell comm | A2A bus with capability discovery | N/A | Message passing |
| Music | MIDI cells, sonification | N/A | N/A |

## License

MIT
