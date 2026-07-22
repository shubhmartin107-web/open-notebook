use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use onb_kernel::dag;
use onb_kernel::notebook::{CellKind, Notebook};

fn build_notebook_with_cells(n: usize) -> Notebook {
    let mut nb = Notebook::new("bench");
    for i in 0..n {
        if i == 0 {
            nb.add_cell(CellKind::Python, "x = 1");
        } else if i == 1 {
            nb.add_cell(CellKind::Python, "y = x + 1");
        } else if i == 2 {
            nb.add_cell(CellKind::Sql, "SELECT 1 AS result");
        } else {
            nb.add_cell(CellKind::Python, &format!("z_{} = y + {}", i, i));
        }
    }
    nb
}

fn bench_dag_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("dag_build");
    for size in [10_usize, 50, 100, 500] {
        let nb = build_notebook_with_cells(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &nb, |b, nb| {
            b.iter(|| {
                let mut clone = black_box(nb.clone());
                dag::build_dag(&mut clone).ok();
            });
        });
    }
    group.finish();
}

fn bench_dag_cycle_detection(c: &mut Criterion) {
    let nb = build_notebook_with_cells(10);
    c.bench_function("dag_cycle_check", |b| {
        b.iter(|| {
            let mut clone = black_box(nb.clone());
            dag::build_dag(&mut clone).ok();
        });
    });
}

criterion_group!(benches, bench_dag_build, bench_dag_cycle_detection);
criterion_main!(benches);
