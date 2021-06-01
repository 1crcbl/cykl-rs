// Benchmarks for data structures.

#[allow(unused_imports)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use cykl::tour::TwoLevelList;
use cykl::tour::{Tour, TourImpltor};
use cykl::Repo;
use cykl::Scalar;
use cykl::{tour::Array, RepoBuilder};
use tspf::WeightKind;

const INDEX: usize = 987_654;

#[allow(dead_code)]
pub fn create_repo(n_nodes: usize) -> Repo {
    let mut repo = RepoBuilder::new(WeightKind::Euc2d)
        .capacity(n_nodes)
        .build();
    for ii in 0..n_nodes {
        repo.add(ii as Scalar, ii as Scalar, ii as Scalar);
    }
    repo
}

fn benchmark_get(c: &mut Criterion) {
    let repo = create_repo(1_000_000);

    let arr = Array::new(&repo);
    let arre: TourImpltor = Array::new(&repo).into();
    let tll = TwoLevelList::with_default_order(&repo, 100);
    let tlle: TourImpltor = TwoLevelList::with_default_order(&repo, 100).into();

    c.bench_function("Array Get", |b| b.iter(|| arr.get(black_box(INDEX - 1))));
    c.bench_function("TLL Get", |b| b.iter(|| tll.get(black_box(INDEX - 1))));
    c.bench_function("Array (ED) 1M Get", |b| {
        b.iter(|| arre.get(black_box(INDEX - 1)))
    });
    c.bench_function("TLL (ED) 1M Get", |b| {
        b.iter(|| tlle.get(black_box(INDEX - 1)))
    });
}

fn benchmark_successor(c: &mut Criterion) {
    let repo = create_repo(1_000_000);

    let arr = Array::new(&repo);
    let arre: TourImpltor = Array::new(&repo).into();
    let tll = TwoLevelList::with_default_order(&repo, 100);
    let tlle: TourImpltor = TwoLevelList::with_default_order(&repo, 100).into();

    c.bench_function("Array Successor", |b| {
        b.iter(|| arr.successor_at(black_box(INDEX - 1)))
    });
    c.bench_function("TLL Successor", |b| {
        b.iter(|| tll.successor_at(black_box(INDEX - 1)))
    });
    c.bench_function("Array (ED) Successor", |b| {
        b.iter(|| arre.successor_at(black_box(INDEX - 1)))
    });
    c.bench_function("TLL (ED) Successor", |b| {
        b.iter(|| tlle.successor_at(black_box(INDEX - 1)))
    });
}

fn benchmark_predecessor(c: &mut Criterion) {
    let repo = create_repo(1_000_000);

    let arr = Array::new(&repo);
    let arre: TourImpltor = Array::new(&repo).into();
    let tll = TwoLevelList::with_default_order(&repo, 100);
    let tlle: TourImpltor = TwoLevelList::with_default_order(&repo, 100).into();

    c.bench_function("Array Predecessor", |b| {
        b.iter(|| arr.get(black_box(INDEX - 1)))
    });
    c.bench_function("TLL Predecessor", |b| {
        b.iter(|| tll.get(black_box(INDEX - 1)))
    });
    c.bench_function("Array (ED) Predecessor", |b| {
        b.iter(|| arre.get(black_box(INDEX - 1)))
    });
    c.bench_function("TLL (ED) Predecessor", |b| {
        b.iter(|| tlle.get(black_box(INDEX - 1)))
    });
}

fn benchmark_between(c: &mut Criterion) {
    let repo = create_repo(1_000_000);

    let arr = Array::new(&repo);
    let arre: TourImpltor = Array::new(&repo).into();
    let tll = TwoLevelList::with_default_order(&repo, 100);
    let tlle: TourImpltor = TwoLevelList::with_default_order(&repo, 100).into();

    c.bench_function("Array Between", |b| {
        b.iter(|| arr.between_at(black_box(1), black_box(500_000), black_box(1_000_000 - 1)))
    });
    c.bench_function("TLL Between", |b| {
        b.iter(|| tll.between_at(black_box(1), black_box(500_000), black_box(1_000_000 - 1)))
    });
    c.bench_function("Array (ED) Between", |b| {
        b.iter(|| arre.between_at(black_box(1), black_box(500_000), black_box(1_000_000 - 1)))
    });
    c.bench_function("TLL (ED) Between", |b| {
        b.iter(|| tlle.between_at(black_box(1), black_box(500_000), black_box(1_000_000 - 1)))
    });
}

/// Flip an entire segment (99 elements).
fn benchmark_flip_case_1(c: &mut Criterion) {
    fn flip(tour: &mut impl Tour) {
        let left = black_box(0);
        let next_left = tour.successor_at(left).unwrap().index();
        let next_right = 100;
        let right = tour.predecessor_at(next_right).unwrap().index();
        tour.flip_at(left, next_left, right, next_right);
    }

    let repo = create_repo(1_000_000);

    let mut arr = Array::new(&repo);
    let mut arre: TourImpltor = Array::new(&repo).into();
    let mut tll = TwoLevelList::with_default_order(&repo, 100);
    let mut tlle: TourImpltor = TwoLevelList::with_default_order(&repo, 100).into();

    c.bench_function("Array Flip - Case 1", |b| b.iter(|| flip(&mut arr)));
    c.bench_function("TLL   Flip - Case 1", |b| b.iter(|| flip(&mut tll)));
    c.bench_function("Array (ED) Flip - Case 1", |b| b.iter(|| flip(&mut arre)));
    c.bench_function("TLL   (ED) Flip - Case 1", |b| b.iter(|| flip(&mut tlle)));
}

/// Flip an entire segment (100 elements).
fn benchmark_flip_case_2(c: &mut Criterion) {
    fn flip(tour: &mut impl Tour) {
        let left = black_box(99);
        let next_left = tour.successor_at(left).unwrap().index();
        let next_right = 200;
        let right = tour.predecessor_at(next_right).unwrap().index();
        tour.flip_at(left, next_left, right, next_right);
    }

    let repo = create_repo(1_000_000);

    let mut arr = Array::new(&repo);
    let mut arre: TourImpltor = Array::new(&repo).into();
    let mut tll = TwoLevelList::with_default_order(&repo, 100);
    let mut tlle: TourImpltor = TwoLevelList::with_default_order(&repo, 100).into();

    c.bench_function("Array Flip - Case 2", |b| b.iter(|| flip(&mut arr)));
    c.bench_function("TLL   Flip - Case 2", |b| b.iter(|| flip(&mut tll)));
    c.bench_function("Array (ED) Flip - Case 2", |b| b.iter(|| flip(&mut arre)));
    c.bench_function("TLL   (ED) Flip - Case 2", |b| b.iter(|| flip(&mut tlle)));
}

/// Flip multiple segments.
fn benchmark_flip_case_3(c: &mut Criterion) {
    fn flip(tour: &mut impl Tour) {
        let left = black_box(99);
        let next_left = tour.successor_at(left).unwrap().index();
        let next_right = 1000;
        let right = tour.predecessor_at(next_right).unwrap().index();
        tour.flip_at(left, next_left, right, next_right);
    }

    let repo = create_repo(1_000_000);

    let mut arr = Array::new(&repo);
    let mut arre: TourImpltor = Array::new(&repo).into();
    let mut tll = TwoLevelList::with_default_order(&repo, 100);
    let mut tlle: TourImpltor = TwoLevelList::with_default_order(&repo, 100).into();

    c.bench_function("Array Flip - Case 3", |b| b.iter(|| flip(&mut arr)));
    c.bench_function("TLL   Flip - Case 3", |b| b.iter(|| flip(&mut tll)));
    c.bench_function("Array (ED) Flip - Case 3", |b| b.iter(|| flip(&mut arre)));
    c.bench_function("TLL   (ED) Flip - Case 3", |b| b.iter(|| flip(&mut tlle)));
}

criterion_group!(
    benches,
    benchmark_get,
    benchmark_successor,
    benchmark_predecessor,
    benchmark_between,
    benchmark_flip_case_1,
    benchmark_flip_case_2,
    benchmark_flip_case_3,
);
criterion_main!(benches);
