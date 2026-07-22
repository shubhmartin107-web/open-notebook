use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use onb_kernel::notebook::format;
use onb_kernel::notebook::{CellKind, Notebook};

fn build_notebook_with_cells(n: usize) -> Notebook {
    let mut nb = Notebook::new("bench");
    for i in 0..n {
        match i % 3 {
            0 => {
                nb.add_cell(CellKind::Python, &format!("x_{} = {}", i, i));
            }
            1 => {
                nb.add_cell(CellKind::Sql, &format!("SELECT {} AS val", i));
            }
            _ => {
                nb.add_cell(CellKind::Markdown, &format!("# Cell {}", i));
            }
        }
    }
    nb
}

fn bench_save_and_load(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let mut group = c.benchmark_group("format_roundtrip");

    for size in [1, 10, 50] {
        let nb = build_notebook_with_cells(size);
        let path = dir.path().join(format!("bench_{}.onb", size));

        group.bench_with_input(BenchmarkId::new("save", size), &(&nb, &path), |b, (nb, path)| {
            b.iter(|| format::save_to_file(black_box(nb), black_box(path)));
        });

        format::save_to_file(&nb, &path).unwrap();

        group.bench_with_input(BenchmarkId::new("load", size), &path, |b, path| {
            b.iter(|| format::load_from_file(black_box(path)));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_save_and_load);
criterion_main!(benches);
