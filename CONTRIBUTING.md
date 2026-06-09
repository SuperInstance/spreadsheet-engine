# Contributing to spreadsheet-engine

Thank you for your interest in improving the living AI spreadsheet engine!

## Getting Started

```bash
git clone https://github.com/SuperInstance/spreadsheet-engine.git
cd spreadsheet-engine
cargo build
cargo test
```

## Development Workflow

1. **Fork** the repository
2. **Create a branch**: `git checkout -b feature/your-feature`
3. **Make changes** with tests
4. **Run checks**: `cargo test && cargo clippy && cargo fmt --check`
5. **Commit**: follow [Conventional Commits](https://www.conventionalcommits.org/)
6. **Push** and open a Pull Request

## Architecture Decisions

### Why a sparse grid instead of a dense 2D array?

Most spreadsheets are 99% empty. A `HashMap<CellId, Cell>` gives O(1) lookup with minimal memory overhead. We don't pay for cells that don't exist.

### Why topological sort for evaluation?

Cells form a dependency DAG. Topological sort (Kahn's algorithm) ensures each cell is evaluated only after its dependencies. Cycle detection is built into the `would_create_cycle` method — you can check before adding a dependency.

### Why γ + η = budget?

This is the conservation law from Noether's theorem: if the system has budget symmetry (the total resources don't change), the budget itself is conserved. Each agent cell tracks γ (compute spend) and η (memory usage). The ConservationMonitor sums across all agents and checks against the total budget.

### Why 7 cell types?

Each cell type maps to a fundamentally different computation model:
- **Value**: static data
- **Agent**: autonomous computation with budget
- **Training**: iterative optimization with loss tracking
- **Simulation**: state evolution with tick counters
- **A2A**: communication endpoint
- **MIDI**: audio output
- **Formula**: aggregation/transformation

Adding a new cell type means: implement the `Cell` enum variant, add evaluation logic in `engine_internal::evaluate_cell`, and update `type_name()`.

## How to Add a New Cell Type

1. Define the cell struct in its own module (e.g., `src/new_cell.rs`)
2. Add it as a variant in the `Cell` enum in `src/cell.rs`
3. Add the `state()` and `type_name()` match arms
4. Add evaluation logic in `src/engine.rs` → `engine_internal::evaluate_cell`
5. Add the module declaration and re-export in `src/lib.rs`
6. Write tests!

## How to Add a New Formula Operation

1. Add the variant to `FormulaOp` in `src/formula.rs`
2. Implement the `Display` match arm
3. Add evaluation logic in `FormulaCell::evaluate`
4. Write tests in the `mod tests` block

## Testing

```bash
cargo test                    # All tests
cargo test -- --nocapture     # With println output
cargo test test_formula       # Just formula tests
```

All new code must have tests. Aim for >90% coverage on new modules.

## Benchmarking

```bash
cargo bench  # If criterion benchmarks exist
```

For performance-critical changes, benchmark before and after. Key metrics:
- Tick latency for grids with 100, 1000, 10000 cells
- Conservation check time vs number of agent cells
- Topological sort time vs grid size

## Code Style

- `cargo fmt` — no debate
- `cargo clippy` — warnings are errors in CI
- Doc comments on all `pub` items
- `# Example` sections in doc comments for core types

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):
- `feat:` new features
- `fix:` bug fixes
- `docs:` documentation changes
- `refactor:` code restructuring
- `test:` test additions/changes
- `chore:` maintenance tasks

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
