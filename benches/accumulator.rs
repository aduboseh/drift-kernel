//! Drift Kernel Benchmarks
//!
//! Run with: `cargo bench`
//!
//! These benchmarks measure real performance on your hardware.
//! Results vary by CPU, compiler version, and optimization flags.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use drift_kernel::{neumaier_sum_slice, ScgAccumulator};

fn bench_accumulator_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("accumulator_add");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut acc = ScgAccumulator::new(0.0);
                for i in 0..size {
                    acc.add(black_box(i as f64 * 0.1));
                }
                acc.total()
            });
        });
    }

    group.finish();
}

fn bench_neumaier_sum_slice(c: &mut Criterion) {
    let mut group = c.benchmark_group("neumaier_sum_slice");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        let values: Vec<f64> = (0..*size).map(|i| i as f64 * 0.1).collect();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &values, |b, values| {
            b.iter(|| neumaier_sum_slice(black_box(values)));
        });
    }

    group.finish();
}

fn bench_naive_sum_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("naive_vs_compensated");

    let size = 10_000;
    let values: Vec<f64> = (0..size).map(|i| i as f64 * 0.1).collect();

    group.throughput(Throughput::Elements(size as u64));

    group.bench_function("naive_iter_sum", |b| {
        b.iter(|| {
            let sum: f64 = black_box(&values).iter().sum();
            sum
        });
    });

    group.bench_function("neumaier_compensated", |b| {
        b.iter(|| neumaier_sum_slice(black_box(&values)));
    });

    group.bench_function("accumulator_loop", |b| {
        b.iter(|| {
            let mut acc = ScgAccumulator::new(0.0);
            for &v in black_box(&values) {
                acc.add(v);
            }
            acc.total()
        });
    });

    group.finish();
}

fn bench_adversarial_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("adversarial");

    // Alternating huge/tiny - worst case for naive summation
    group.bench_function("alternating_magnitude_10k", |b| {
        b.iter(|| {
            let mut acc = ScgAccumulator::new(0.0);
            for _ in 0..10_000 {
                acc.add(black_box(1e15));
                acc.add(black_box(1e-15));
                acc.add(black_box(-1e15));
                acc.add(black_box(-1e-15));
            }
            acc.total()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_accumulator_add,
    bench_neumaier_sum_slice,
    bench_naive_sum_comparison,
    bench_adversarial_workload
);
criterion_main!(benches);
