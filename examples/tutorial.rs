//! Step-by-step tutorial: build a spreadsheet with agents, formulas, and conservation.
//!
//! Run with: cargo run --example tutorial

use spreadsheet_engine::{
    Grid, Engine, Cell,
    cell::{CellId, ValueCell, AgentCell},
    formula::{FormulaCell, FormulaOp},
};

fn main() {
    println!("=== Spreadsheet Engine Tutorial ===\n");

    // Step 1: Create a grid with a conservation budget
    println!("Step 1: Create a grid with budget = 2.0");
    let mut grid = Grid::with_budget(2.0, 0.01);
    println!("  Total budget: {}", grid.total_budget);
    println!("  Tolerance: {}", grid.tolerance);

    // Step 2: Add plain value cells
    println!("\nStep 2: Add value cells");
    let a1 = CellId::new(0, 0);
    let b1 = CellId::new(0, 1);
    grid.insert(a1, Cell::Value(ValueCell::from(42.0)));
    grid.insert(b1, Cell::Value(ValueCell::text("hello")));
    println!("  A1 = 42.0");
    println!("  B1 = \"hello\"");

    // Step 3: Add an AI agent with capabilities
    println!("\nStep 3: Add an agent cell");
    let c1 = CellId::new(0, 2);
    let agent = AgentCell::new("code-agent", 1.0)
        .with_capability("rust", 0.95)
        .with_capability("python", 0.80)
        .with_capability("math", 0.90);
    grid.insert(c1, Cell::Agent(agent));
    println!("  C1 = Agent 'code-agent' with 3 capabilities");
    println!("  Budget: 1.0");

    // Step 4: Add a formula cell that sums A1 and C1
    println!("\nStep 4: Add a SUM formula");
    let d1 = CellId::new(0, 3);
    let formula = FormulaCell::new(FormulaOp::Sum, vec![a1, c1]);
    grid.insert(d1, Cell::Formula(formula));
    grid.add_dependency(d1, a1); // D1 depends on A1
    grid.add_dependency(d1, c1); // D1 depends on C1
    println!("  D1 = SUM(A1, C1)");
    println!("  Dependencies: D1 → A1, D1 → C1");

    // Step 5: Run the engine
    println!("\nStep 5: Run engine ticks");
    let mut engine = Engine::new(grid);
    engine.run_ticks(5).unwrap();
    println!("  Ran {} ticks", engine.tick);

    // Step 6: Read results
    println!("\nStep 6: Results");
    for (id, label) in [(a1, "A1"), (b1, "B1"), (c1, "C1"), (d1, "D1")] {
        if let Some(val) = engine.value(&id) {
            println!("  {} = {}", label, val);
        }
    }

    // Step 7: Check conservation health
    println!("\nStep 7: Conservation health");
    println!("  Health: {:.1}%", engine.health() * 100.0);

    // Step 8: Check evaluation order
    println!("\nStep 8: Evaluation order");
    let order = engine.grid.eval_order().unwrap();
    let labels: Vec<String> = order.iter().map(|id| format!("{}", id)).collect();
    println!("  Order: {}", labels.join(" → "));
}
