// Benchmarks for data structures.

#[allow(unused_imports)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use cykl_rs::metric::MetricKind;
use cykl_rs::node::Container;
use cykl_rs::tour::Array;
use cykl_rs::tour::Tour;
use cykl_rs::tour::TwoLevelTree;
use cykl_rs::Scalar;

#[allow(dead_code)]
pub fn create_container(n_nodes: usize) -> Container {
    let mut container = Container::new(MetricKind::Euc2d);
    for ii in 0..n_nodes {
        container.add(ii as Scalar, ii as Scalar, ii as Scalar);
    }
    container
}

fn benchmark_next(c: &mut Criterion) {
    let container_1mil = create_container(1_000_000);
    let arr_1mil = Array::new(&container_1mil);
    let tlt_1mil = TwoLevelTree::with_default_order(&container_1mil, 100);
    c.bench_function("Array 1M Next", |b| {
        b.iter(|| arr_1mil.next_at(black_box(1_000_000 - 1)))
    });
    c.bench_function("TLT   1M Next", |b| {
        b.iter(|| tlt_1mil.next_at(black_box(1_000_000 - 1)))
    });
}

fn benchmark_prev(c: &mut Criterion) {
    let container_1mil = create_container(1_000_000);
    let arr_1mil = Array::new(&container_1mil);
    let tlt_1mil = TwoLevelTree::with_default_order(&container_1mil, 100);
    c.bench_function("Array 1M Prev", |b| {
        b.iter(|| arr_1mil.prev_at(black_box(1_000_000 - 1)))
    });
    c.bench_function("TLT   1M Prev", |b| {
        b.iter(|| tlt_1mil.prev_at(black_box(1_000_000 - 1)))
    });
}

fn benchmark_between(c: &mut Criterion) {
    let container_1mil = create_container(1_000_000);
    let arr_1mil = Array::new(&container_1mil);
    let tlt_1mil = TwoLevelTree::with_default_order(&container_1mil, 100);
    c.bench_function("Array 1M Between", |b| {
        b.iter(|| arr_1mil.between_at(black_box(1), black_box(500_000), black_box(1_000_000 - 1)))
    });
    c.bench_function("TLT   1M Between", |b| {
        b.iter(|| tlt_1mil.between_at(black_box(1), black_box(500_000), black_box(1_000_000 - 1)))
    });
}

criterion_group!(benches, benchmark_next, benchmark_prev, benchmark_between);
criterion_main!(benches);
