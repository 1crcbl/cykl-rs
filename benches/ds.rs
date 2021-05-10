// Benchmarks for data structures.

#[allow(unused_imports)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use cykl_rs::metric::MetricKind;
use cykl_rs::node::Container;
use cykl_rs::tour::Array;
use cykl_rs::tour::Tour;
use cykl_rs::tour::TwoLevelTree;
use cykl_rs::tour::Vertex;
use cykl_rs::Scalar;

use cykl_rs::tour::tll::TwoLevelList;

const INDEX: usize = 987_654;

#[allow(dead_code)]
pub fn create_container(n_nodes: usize) -> Container {
    let mut container = Container::new(MetricKind::Euc2d);
    for ii in 0..n_nodes {
        container.add(ii as Scalar, ii as Scalar, ii as Scalar);
    }
    container
}

fn benchmark_get(c: &mut Criterion) {
    let container_1mil = create_container(1_000_000);
    let arr_1mil = Array::new(&container_1mil);
    let tlt_1mil = TwoLevelTree::with_default_order(&container_1mil, 100);
    let tll_1mil = TwoLevelList::with_default_order(&container_1mil, 100);
    c.bench_function("Array 1M Get", |b| {
        b.iter(|| arr_1mil.get(black_box(INDEX - 1)))
    });
    c.bench_function("TLT   1M Get", |b| {
        b.iter(|| tlt_1mil.get(black_box(INDEX - 1)))
    });
    c.bench_function("TLL   1M Get", |b| {
        b.iter(|| tll_1mil.get(black_box(INDEX - 1)))
    });
}

fn benchmark_successor(c: &mut Criterion) {
    let container_1mil = create_container(1_000_000);
    let arr_1mil = Array::new(&container_1mil);
    let tlt_1mil = TwoLevelTree::with_default_order(&container_1mil, 100);
    let tll_1mil = TwoLevelList::with_default_order(&container_1mil, 100);
    c.bench_function("Array 1M Successor", |b| {
        b.iter(|| arr_1mil.successor_at(black_box(INDEX - 1)))
    });
    c.bench_function("TLT   1M Successor", |b| {
        b.iter(|| tlt_1mil.successor_at(black_box(INDEX - 1)))
    });
    c.bench_function("TLL   1M Successor", |b| {
        b.iter(|| tll_1mil.successor_at(black_box(INDEX - 1)))
    });
}

fn benchmark_predecessor(c: &mut Criterion) {
    let container_1mil = create_container(1_000_000);
    let arr_1mil = Array::new(&container_1mil);
    let tlt_1mil = TwoLevelTree::with_default_order(&container_1mil, 100);
    let tll_1mil = TwoLevelList::with_default_order(&container_1mil, 100);
    c.bench_function("Array 1M Predecessor", |b| {
        b.iter(|| arr_1mil.get(black_box(INDEX - 1)))
    });
    c.bench_function("TLT   1M Predecessor", |b| {
        b.iter(|| tlt_1mil.get(black_box(INDEX - 1)))
    });
    c.bench_function("TLL   1M Predecessor", |b| {
        b.iter(|| tll_1mil.get(black_box(INDEX - 1)))
    });
}

fn benchmark_between(c: &mut Criterion) {
    let container_1mil = create_container(1_000_000);
    let arr_1mil = Array::new(&container_1mil);
    let tlt_1mil = TwoLevelTree::with_default_order(&container_1mil, 100);
    let tll_1mil = TwoLevelList::with_default_order(&container_1mil, 100);
    c.bench_function("Array 1M Between", |b| {
        b.iter(|| arr_1mil.between_at(black_box(1), black_box(500_000), black_box(1_000_000 - 1)))
    });
    c.bench_function("TLT   1M Between", |b| {
        b.iter(|| tlt_1mil.between_at(black_box(1), black_box(500_000), black_box(1_000_000 - 1)))
    });
    c.bench_function("TLL   1M Between", |b| {
        b.iter(|| tll_1mil.between_at(black_box(1), black_box(500_000), black_box(1_000_000 - 1)))
    });
}

fn flip(tour: &mut impl Tour) {
    let left = black_box(10);
    let next_left = tour.successor_at(left).unwrap().index();
    let right = left + 500_000;
    let next_right = tour.successor_at(right).unwrap().index();
    tour.flip_at(left, next_left, right, next_right);
}

fn benchmark_flip(c: &mut Criterion) {
    let container_1mil = create_container(1_000_000);
    let mut arr_1mil = Array::new(&container_1mil);
    let mut tlt_1mil = TwoLevelTree::with_default_order(&container_1mil, 100);
    c.bench_function("Array 1M Flip", |b| b.iter(|| flip(&mut arr_1mil)));
    c.bench_function("TLT   1M Flip", |b| b.iter(|| flip(&mut tlt_1mil)));
}

criterion_group!(
    benches,
    benchmark_get,
    benchmark_successor,
    benchmark_predecessor,
    benchmark_between,
    // benchmark_flip
);
criterion_main!(benches);
