use criterion::{criterion_group, criterion_main, Criterion};
use crabquick::Context;

fn bench_context_creation(c: &mut Criterion) {
    c.bench_function("context_new_8kb", |b| {
        b.iter(|| Context::new(8192));
    });
}

fn bench_value_operations(c: &mut Criterion) {
    use crabquick::JSValue;

    c.bench_function("value_int_encoding", |b| {
        b.iter(|| {
            let val = JSValue::from_int(42);
            val.to_int()
        });
    });

    c.bench_function("value_special_creation", |b| {
        b.iter(|| {
            let _null = JSValue::null();
            let _undef = JSValue::undefined();
            let _bool = JSValue::bool(true);
        });
    });
}

criterion_group!(benches, bench_context_creation, bench_value_operations);
criterion_main!(benches);
