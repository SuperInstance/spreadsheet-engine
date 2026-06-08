//! Formula evaluator — standard math + evolutionary operations.
//!
//! The living spreadsheet supports not just `SUM` and `AVERAGE` but also
//! evolutionary formulas from the SuperInstance ecosystem:
//!
//! - `EVOLVE` — genetic optimization over cell values
//! - `SPECIES` — cluster cells by similarity
//! - `PARETO` — find Pareto-optimal cells
//! - `ENTROPY` — measure diversity of a range
//! - `CONSERVE` — track conservation law across cells

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::cell::{CellId, CellState, CellValue};


// ── Formula Operations ───────────────────────────────────────────────

/// Operations a formula cell can perform.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FormulaOp {
    /// Arithmetic: ADD, SUB, MUL, DIV.
    Add,
    Sub,
    Mul,
    Div,
    /// Sum a range of cells.
    Sum,
    /// Average of a range.
    Average,
    /// Count non-empty cells.
    Count,
    /// Maximum value in range.
    Max,
    /// Minimum value in range.
    Min,
    /// No-op (identity).
    Identity,
    /// Genetic optimization: mutate cell values, select fittest, repeat.
    /// Params: generations, population_size, mutation_rate.
    Evolve { generations: u32, population_size: usize, mutation_rate: f64 },
    /// Cluster cells by similarity (k-means over ternary vectors).
    Species { k: usize },
    /// Find Pareto-optimal cells (multi-objective).
    Pareto,
    /// Shannon entropy of a range (diversity measure).
    Entropy,
    /// Track conservation: verify γ + η = budget across cells.
    Conserve,
}

impl std::fmt::Display for FormulaOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FormulaOp::Add => write!(f, "ADD"),
            FormulaOp::Sub => write!(f, "SUB"),
            FormulaOp::Mul => write!(f, "MUL"),
            FormulaOp::Div => write!(f, "DIV"),
            FormulaOp::Sum => write!(f, "SUM"),
            FormulaOp::Average => write!(f, "AVERAGE"),
            FormulaOp::Count => write!(f, "COUNT"),
            FormulaOp::Max => write!(f, "MAX"),
            FormulaOp::Min => write!(f, "MIN"),
            FormulaOp::Evolve { .. } => write!(f, "EVOLVE"),
            FormulaOp::Species { .. } => write!(f, "SPECIES"),
            FormulaOp::Pareto => write!(f, "PARETO"),
            FormulaOp::Entropy => write!(f, "ENTROPY"),
            FormulaOp::Conserve => write!(f, "CONSERVE"),
            FormulaOp::Identity => write!(f, "IDENTITY"),
        }
    }
}

// ── Formula Cell ─────────────────────────────────────────────────────

/// A cell containing a formula that operates over a range of inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaCell {
    pub operation: FormulaOp,
    /// Input cell IDs this formula reads from.
    pub inputs: Vec<CellId>,
    /// Cached result from last evaluation.
    pub cached: CellValue,
    pub state: CellState,
}

impl FormulaCell {
    pub fn new(op: FormulaOp, inputs: Vec<CellId>) -> Self {
        Self { operation: op, inputs, cached: CellValue::Empty, state: CellState::Idle }
    }

    /// Evaluate this formula given the resolved input values.
    pub fn evaluate(&mut self, values: &[CellValue]) -> CellValue {
        let result = match &self.operation {
            FormulaOp::Add => arithmetic(values, |a, b| a + b),
            FormulaOp::Sub => arithmetic(values, |a, b| a - b),
            FormulaOp::Mul => arithmetic(values, |a, b| a * b),
            FormulaOp::Div => arithmetic(values, |a, b| {
                if b.abs() < 1e-12 { f64::NAN } else { a / b }
            }),
            FormulaOp::Sum => {
                let sum: f64 = values.iter().filter_map(|v| v.as_f64()).sum();
                CellValue::Number(sum)
            }
            FormulaOp::Average => {
                let nums: Vec<f64> = values.iter().filter_map(|v| v.as_f64()).collect();
                if nums.is_empty() { CellValue::Error("AVERAGE: no numbers".into()) }
                else { CellValue::Number(nums.iter().sum::<f64>() / nums.len() as f64) }
            }
            FormulaOp::Count => CellValue::Number(values.iter().filter(|v| !v.is_empty()).count() as f64),
            FormulaOp::Max => {
                let nums: Vec<f64> = values.iter().filter_map(|v| v.as_f64()).collect();
                CellValue::Number(nums.iter().copied().fold(f64::NEG_INFINITY, f64::max))
            }
            FormulaOp::Min => {
                let nums: Vec<f64> = values.iter().filter_map(|v| v.as_f64()).collect();
                CellValue::Number(nums.iter().copied().fold(f64::INFINITY, f64::min))
            }
            FormulaOp::Identity => {
                values.first().cloned().unwrap_or(CellValue::Empty)
            }
            FormulaOp::Entropy => compute_entropy(values),
            FormulaOp::Pareto => compute_pareto(values),
            FormulaOp::Evolve { generations, population_size, mutation_rate } => {
                simulate_evolve(values, *generations, *population_size, *mutation_rate)
            }
            FormulaOp::Species { k } => simulate_species(values, *k),
            FormulaOp::Conserve => compute_conserve(values),
        };
        self.cached = result.clone();
        self.state = CellState::Ready;
        result
    }
}

// ── Arithmetic helper ────────────────────────────────────────────────

fn arithmetic(values: &[CellValue], op: impl Fn(f64, f64) -> f64) -> CellValue {
    let nums: Vec<f64> = values.iter().filter_map(|v| v.as_f64()).collect();
    if nums.len() < 2 {
        return CellValue::Error("Need at least 2 values".into());
    }
    let result = nums[1..].iter().fold(nums[0], |acc, &v| op(acc, v));
    CellValue::Number(result)
}

// ── Entropy ──────────────────────────────────────────────────────────

/// Shannon entropy: H = -Σ p_i * log2(p_i).
fn compute_entropy(values: &[CellValue]) -> CellValue {
    let nums: Vec<f64> = values.iter().filter_map(|v| v.as_f64()).collect();
    if nums.is_empty() { return CellValue::Number(0.0); }

    // Discretize into bins for frequency counting
    let mut counts: HashMap<u64, usize> = HashMap::new();
    for &n in &nums {
        let bin = (n * 100.0).round() as u64; // bin by 0.01 precision
        *counts.entry(bin).or_insert(0) += 1;
    }
    let total = nums.len() as f64;
    let entropy: f64 = counts.values()
        .map(|&c| {
            let p = c as f64 / total;
            -p * p.log2()
        })
        .sum();
    CellValue::Number(entropy)
}

// ── Pareto front ─────────────────────────────────────────────────────

/// Find Pareto-optimal points (non-dominated).
/// For simplicity, treats pairs of consecutive values as (x, y) objectives.
fn compute_pareto(values: &[CellValue]) -> CellValue {
    let nums: Vec<f64> = values.iter().filter_map(|v| v.as_f64()).collect();
    if nums.len() < 2 { return CellValue::Number(0.0); }

    // Pair consecutive values as objectives
    let points: Vec<[f64; 2]> = nums.chunks(2)
        .filter_map(|chunk| {
            if chunk.len() == 2 { Some([chunk[0], chunk[1]]) } else { None }
        })
        .collect();

    let mut pareto_count = 0;
    for (i, p) in points.iter().enumerate() {
        let dominated = points.iter().enumerate().any(|(j, q)| {
            j != i && q[0] >= p[0] && q[1] >= p[1] && (q[0] > p[0] || q[1] > p[1])
        });
        if !dominated { pareto_count += 1; }
    }
    CellValue::Number(pareto_count as f64)
}

// ── Evolution simulation ─────────────────────────────────────────────

/// Simple genetic optimization: random mutations, keep fittest.
/// Fitness = sum of absolute values (maximize total energy).
fn simulate_evolve(values: &[CellValue], generations: u32, pop_size: usize, mutation_rate: f64) -> CellValue {
    let nums: Vec<f64> = values.iter().filter_map(|v| v.as_f64()).collect();
    if nums.is_empty() { return CellValue::Vector(vec![]); }

    let _n = nums.len();
    let mut population: Vec<Vec<f64>> = (0..pop_size)
        .map(|_| nums.iter().map(|&v| v + (rand_simple() - 0.5) * 2.0).collect())
        .collect();

    for _ in 0..generations {
        // Sort by fitness (sum of absolute values)
        population.sort_by(|a, b| {
            let fa: f64 = a.iter().map(|x| x.abs()).sum();
            let fb: f64 = b.iter().map(|x| x.abs()).sum();
            fb.partial_cmp(&fa).unwrap_or(std::cmp::Ordering::Equal)
        });
        // Keep top 50%
        population.truncate(pop_size / 2);
        // Reproduce with mutation
        let parents = population.clone();
        while population.len() < pop_size {
            let parent = &parents[rand_simple_index(parents.len())];
            let child: Vec<f64> = parent.iter().map(|&v| {
                if rand_simple() < mutation_rate {
                    v + (rand_simple() - 0.5) * 2.0
                } else {
                    v
                }
            }).collect();
            population.push(child);
        }
    }

    // Return fittest
    population.sort_by(|a, b| {
        let fa: f64 = a.iter().map(|x| x.abs()).sum();
        let fb: f64 = b.iter().map(|x| x.abs()).sum();
        fb.partial_cmp(&fa).unwrap_or(std::cmp::Ordering::Equal)
    });
    CellValue::Vector(population[0].clone())
}

// ── Species clustering ───────────────────────────────────────────────

/// Simple k-means-ish clustering (count distinct species).
fn simulate_species(values: &[CellValue], k: usize) -> CellValue {
    let nums: Vec<f64> = values.iter().filter_map(|v| v.as_f64()).collect();
    if nums.is_empty() || k == 0 { return CellValue::Number(0.0); }

    // Very simple: divide the range into k equal bins
    let min = nums.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;
    if range < 1e-12 { return CellValue::Number(1.0); }

    let mut bin_counts = vec![0usize; k];
    for &n in &nums {
        let bin = ((n - min) / range * (k as f64 - 1.0)).round() as usize;
        let bin = bin.min(k - 1);
        bin_counts[bin] += 1;
    }
    let species_count = bin_counts.iter().filter(|&&c| c > 0).count();
    CellValue::Number(species_count as f64)
}

// ── Conservation tracking ────────────────────────────────────────────

/// Track γ + η = budget across cell values.
/// Treats triplets of values as (gamma, eta, budget) and checks conservation.
fn compute_conserve(values: &[CellValue]) -> CellValue {
    let nums: Vec<f64> = values.iter().filter_map(|v| v.as_f64()).collect();
    if nums.len() < 3 { return CellValue::Number(1.0); } // nothing to check = conserved

    let triplets: Vec<[f64; 3]> = nums.chunks(3)
        .filter_map(|c| {
            if c.len() == 3 { Some([c[0], c[1], c[2]]) } else { None }
        })
        .collect();

    if triplets.is_empty() { return CellValue::Number(1.0); }

    let mut total_error = 0.0;
    for [gamma, eta, budget] in &triplets {
        total_error += (gamma + eta - budget).abs();
    }
    let avg_error = total_error / triplets.len() as f64;
    // Health = 1 - normalized_error (clamp 0-1)
    let health = 1.0 - (avg_error / triplets.iter().map(|[_, _, b]| *b).fold(0.0_f64, f64::max).max(0.01)).min(1.0);
    CellValue::Number(health)
}

// ── Simple RNG (no external dep) ─────────────────────────────────────

fn rand_simple() -> f64 {
    
    let nanos = (fastrand::f64() * 1_000_000_000.0) as u32;
    // Simple hash-based PRNG
    let x = nanos.wrapping_mul(1103515245).wrapping_add(12345);
    (x % 10000) as f64 / 10000.0
}

fn rand_simple_index(len: usize) -> usize {
    (rand_simple() * len as f64) as usize % len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formula_add() {
        let mut f = FormulaCell::new(FormulaOp::Add, vec![CellId::new(0, 0), CellId::new(0, 1)]);
        let result = f.evaluate(&[CellValue::Number(3.0), CellValue::Number(4.0)]);
        assert_eq!(result, CellValue::Number(7.0));
    }

    #[test]
    fn test_formula_mul() {
        let mut f = FormulaCell::new(FormulaOp::Mul, vec![CellId::new(0, 0), CellId::new(0, 1)]);
        let result = f.evaluate(&[CellValue::Number(3.0), CellValue::Number(4.0)]);
        assert_eq!(result, CellValue::Number(12.0));
    }

    #[test]
    fn test_formula_div() {
        let mut f = FormulaCell::new(FormulaOp::Div, vec![CellId::new(0, 0), CellId::new(0, 1)]);
        let result = f.evaluate(&[CellValue::Number(10.0), CellValue::Number(2.0)]);
        assert_eq!(result, CellValue::Number(5.0));
    }

    #[test]
    fn test_formula_div_by_zero() {
        let mut f = FormulaCell::new(FormulaOp::Div, vec![CellId::new(0, 0), CellId::new(0, 1)]);
        let result = f.evaluate(&[CellValue::Number(10.0), CellValue::Number(0.0)]);
        assert!(result.is_error() || result.as_f64().map_or(false, |v| v.is_nan()));
    }

    #[test]
    fn test_formula_sum() {
        let mut f = FormulaCell::new(FormulaOp::Sum, vec![]);
        let result = f.evaluate(&[CellValue::Number(1.0), CellValue::Number(2.0), CellValue::Number(3.0)]);
        assert_eq!(result, CellValue::Number(6.0));
    }

    #[test]
    fn test_formula_average() {
        let mut f = FormulaCell::new(FormulaOp::Average, vec![]);
        let result = f.evaluate(&[CellValue::Number(2.0), CellValue::Number(4.0)]);
        assert_eq!(result, CellValue::Number(3.0));
    }

    #[test]
    fn test_formula_count() {
        let mut f = FormulaCell::new(FormulaOp::Count, vec![]);
        let result = f.evaluate(&[CellValue::Number(1.0), CellValue::Empty, CellValue::Number(3.0)]);
        assert_eq!(result, CellValue::Number(2.0)); // counts non-empty
    }

    #[test]
    fn test_formula_max_min() {
        let mut f = FormulaCell::new(FormulaOp::Max, vec![]);
        let result = f.evaluate(&[CellValue::Number(1.0), CellValue::Number(5.0), CellValue::Number(3.0)]);
        assert_eq!(result, CellValue::Number(5.0));

        let mut f2 = FormulaCell::new(FormulaOp::Min, vec![]);
        let result2 = f2.evaluate(&[CellValue::Number(1.0), CellValue::Number(5.0), CellValue::Number(3.0)]);
        assert_eq!(result2, CellValue::Number(1.0));
    }

    #[test]
    fn test_formula_entropy() {
        let mut f = FormulaCell::new(FormulaOp::Entropy, vec![]);
        // All same → entropy = 0
        let result = f.evaluate(&[CellValue::Number(1.0), CellValue::Number(1.0), CellValue::Number(1.0)]);
        assert_eq!(result, CellValue::Number(0.0));
    }

    #[test]
    fn test_formula_entropy_diverse() {
        let mut f = FormulaCell::new(FormulaOp::Entropy, vec![]);
        // Diverse values → entropy > 0
        let result = f.evaluate(&[CellValue::Number(1.0), CellValue::Number(2.0), CellValue::Number(3.0), CellValue::Number(4.0)]);
        if let CellValue::Number(h) = result {
            assert!(h > 0.0, "Diverse values should have positive entropy");
        }
    }

    #[test]
    fn test_formula_pareto() {
        let mut f = FormulaCell::new(FormulaOp::Pareto, vec![]);
        // 3 points: (1,5), (3,3), (5,1) — all Pareto-optimal
        let result = f.evaluate(&[CellValue::Number(1.0), CellValue::Number(5.0), CellValue::Number(3.0), CellValue::Number(3.0), CellValue::Number(5.0), CellValue::Number(1.0)]);
        if let CellValue::Number(n) = result {
            assert!(n >= 1.0, "At least 1 Pareto point");
        }
    }

    #[test]
    fn test_formula_conserve() {
        let mut f = FormulaCell::new(FormulaOp::Conserve, vec![]);
        // (0.3, 0.2, 0.5) → γ+η=budget, conserved
        let result = f.evaluate(&[CellValue::Number(0.3), CellValue::Number(0.2), CellValue::Number(0.5)]);
        if let CellValue::Number(h) = result {
            assert!(h > 0.9, "Should be healthy: {}", h);
        }
    }

    #[test]
    fn test_formula_conserve_violated() {
        let mut f = FormulaCell::new(FormulaOp::Conserve, vec![]);
        // (0.9, 0.8, 0.5) → γ+η >> budget, violated
        let result = f.evaluate(&[CellValue::Number(0.9), CellValue::Number(0.8), CellValue::Number(0.5)]);
        if let CellValue::Number(h) = result {
            assert!(h < 1.0, "Should be unhealthy: {}", h);
        }
    }

    #[test]
    fn test_formula_evolve() {
        let mut f = FormulaCell::new(FormulaOp::Evolve {
            generations: 10, population_size: 20, mutation_rate: 0.3
        }, vec![]);
        let result = f.evaluate(&[CellValue::Number(1.0), CellValue::Number(2.0), CellValue::Number(3.0)]);
        // Should return a vector (the fittest individual)
        assert!(matches!(result, CellValue::Vector(_)));
    }

    #[test]
    fn test_formula_species() {
        let mut f = FormulaCell::new(FormulaOp::Species { k: 3 }, vec![]);
        let result = f.evaluate(&[CellValue::Number(1.0), CellValue::Number(50.0), CellValue::Number(99.0)]);
        if let CellValue::Number(n) = result {
            assert!(n >= 1.0, "Should find at least 1 species");
        }
    }

    #[test]
    fn test_formula_display() {
        assert_eq!(format!("{}", FormulaOp::Add), "ADD");
        assert_eq!(format!("{}", FormulaOp::Evolve { generations: 10, population_size: 20, mutation_rate: 0.3 }), "EVOLVE");
    }

    #[test]
    fn test_formula_cached() {
        let mut f = FormulaCell::new(FormulaOp::Sum, vec![]);
        f.evaluate(&[CellValue::Number(1.0), CellValue::Number(2.0)]);
        assert_eq!(f.cached, CellValue::Number(3.0));
        assert_eq!(f.state, CellState::Ready);
    }
}
