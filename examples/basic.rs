//! Basic spreadsheet-engine usage: create a grid, add cells, run ticks.
//!
//! Run with: cargo run --example basic

use spreadsheet_engine::{
    Grid, Engine, Cell,
    cell::{CellId, ValueCell},
};

fn main() {
    // Create an empty grid
    let mut grid = Grid::new();

    // Add cells at positions (row, col)
    let a1 = CellId::new(0, 0); // A1
    let b1 = CellId::new(0, 1); // B1

    grid.insert(a1, Cell::Value(ValueCell::from(10)));
    grid.insert(b1, Cell::Value(ValueCell::from(20)));

    // Create an engine and run one tick
    let mut engine = Engine::new(grid);
    engine.tick().unwrap();

    // Read values back
    println!("A1 ({}) = {:?}", a1, engine.value(&a1));
    println!("B1 ({}) = {:?}", b1, engine.value(&b1));
    println!("Grid health: {:.1}%", engine.health() * 100.0);
    println!("Cells: {}", engine.cell_count());
}
