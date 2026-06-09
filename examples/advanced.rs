//! Advanced usage: A2A communication, evolutionary formulas, training, and conservation.
//!
//! Run with: cargo run --example advanced

use spreadsheet_engine::{
    Grid, Engine, Cell,
    cell::{CellId, ValueCell, AgentCell, CellValue},
    formula::{FormulaCell, FormulaOp},
    training::TrainingCell,
    simulation::SimulationCell,
    a2a::{A2ABus, A2AMessage, A2AMessageKind},
    conservation::ConservationMonitor,
};

fn main() {
    println!("=== Advanced Spreadsheet Engine ===\n");

    // ── Evolutionary formula: evolve a population ──
    println!("1. Evolutionary Formula");
    let mut grid = Grid::new();
    let a1 = CellId::new(0, 0);
    let b1 = CellId::new(0, 1);
    let c1 = CellId::new(0, 2);
    let d1 = CellId::new(0, 3);

    grid.insert(a1, Cell::Value(ValueCell::from(1.0)));
    grid.insert(b1, Cell::Value(ValueCell::from(2.0)));
    grid.insert(c1, Cell::Value(ValueCell::from(3.0)));

    let evolve = FormulaCell::new(
        FormulaOp::Evolve {
            generations: 50,
            population_size: 30,
            mutation_rate: 0.2,
        },
        vec![a1, b1, c1],
    );
    grid.insert(d1, Cell::Formula(evolve));
    grid.add_dependency(d1, a1);
    grid.add_dependency(d1, b1);
    grid.add_dependency(d1, c1);

    let mut engine = Engine::new(grid);
    engine.tick().unwrap();
    if let Some(CellValue::Vector(fittest)) = engine.value(&d1) {
        println!("   Evolved fittest: {:?}", fittest);
    }

    // ── Training cell ──
    println!("\n2. Training Cell");
    let mut training = TrainingCell::new("resnet-50", 50);
    for _ in 0..10 {
        training.step();
    }
    println!("   Epoch: {}/{}", training.training.current_epoch, training.training.total_epochs);
    println!("   Loss: {:.4}", training.training.loss);
    println!("   Best loss: {:.4}", training.training.best_loss);
    println!("   Progress: {:.0}%", training.progress() * 100.0);

    // ── A2A bus: inter-cell communication ──
    println!("\n3. A2A Communication");
    let mut bus = A2ABus::new();
    let agent_a = CellId::new(0, 0);
    let agent_b = CellId::new(0, 1);

    bus.announce(agent_a, vec!["ml".into(), "rust".into()], 0);
    bus.announce(agent_b, vec!["python".into(), "ml".into()], 0);

    let ml_agents = bus.find_by_capability("ml");
    println!("   Agents with 'ml' capability: {:?}", ml_agents);
    println!("   Total announced: {}", bus.announced().len());
    println!("   Messages queued: {}", bus.total_queued());

    // Send a direct message
    bus.send(A2AMessage {
        from: agent_a, to: agent_b,
        kind: A2AMessageKind::Train,
        payload: CellValue::Text("start training".into()),
        tick: 1,
    });
    let inbox = bus.drain(&agent_b);
    println!("   B's inbox: {} messages", inbox.len());

    // ── Simulation cell ──
    println!("\n4. Simulation Cell");
    let mut sim = SimulationCell::new("fluid-sim", 3)
        .with_state(vec![1.0, 0.5, -0.5]);
    sim.sim.max_ticks = 50;
    for _ in 0..10 {
        sim.step();
    }
    println!("   Progress: {:.0}%", sim.progress() * 100.0);
    println!("   Done: {}", sim.is_done());

    // ── Conservation monitoring ──
    println!("\n5. Conservation Monitoring");
    let mut grid2 = Grid::with_budget(2.0, 0.01);
    let mut agent1 = AgentCell::new("agent-1", 1.0);
    agent1.gamma = 0.6;
    agent1.eta = 0.4;
    grid2.insert(CellId::new(0, 0), Cell::Agent(agent1));

    let mut agent2 = AgentCell::new("agent-2", 1.0);
    agent2.gamma = 0.5;
    agent2.eta = 0.5;
    grid2.insert(CellId::new(0, 1), Cell::Agent(agent2));

    let monitor = ConservationMonitor::new(2.0, 0.01);
    let health = monitor.health(&grid2);
    let violations = monitor.violations(&grid2);
    println!("   Grid health: {:.1}%", health * 100.0);
    println!("   Violations: {} cells", violations.len());
    println!("   Total γ: {:.2}", monitor.total_gamma(&grid2));
    println!("   Total η: {:.2}", monitor.total_eta(&grid2));
}
