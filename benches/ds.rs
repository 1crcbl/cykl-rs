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

fn bench_next(tour: &impl Tour) {
    let len = tour.size();
    for ii in 0..len {
        tour.next_at(ii);
    }
}

fn bench_prev(tour: &impl Tour) {
    let len = tour.size();
    for ii in 0..len {
        tour.prev_at(ii);
    }
}

fn bench_between(tour: &impl Tour) {
    let len = tour.size();
    for _ in 0..len {
        tour.between_at(5, 10, 15);
        tour.between_at(50, 550, 1050);
    }
}

fn array_benchmark(c: &mut Criterion) {
    let container_1mil = create_container(1_000_000);
    let arr_1mil = Array::new(&container_1mil);
    c.bench_function("Array 1M Next", |b| b.iter(|| bench_next(&arr_1mil)));
    c.bench_function("Array 1M Prev", |b| b.iter(|| bench_prev(&arr_1mil)));
    c.bench_function("Array 1M Between", |b| b.iter(|| bench_between(&arr_1mil)));
}

fn tlt_benchmark(c: &mut Criterion) {
    let container_1mil = create_container(1_000_000);
    let tlt_1mil = TwoLevelTree::with_default_order(&container_1mil, 100);
    c.bench_function("TLT 1M Next", |b| b.iter(|| bench_next(&tlt_1mil)));
    c.bench_function("TLT 1M Prev", |b| b.iter(|| bench_prev(&tlt_1mil)));
    c.bench_function("TLT 1M Between", |b| b.iter(|| bench_between(&tlt_1mil)));
}

criterion_group!(benches, array_benchmark, tlt_benchmark);
criterion_main!(benches);
