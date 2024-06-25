use criterion::{criterion_group, criterion_main, Criterion};

use afseq::tidal::Cycle;

// ---------------------------------------------------------------------------------------------

pub fn parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("Complex Cycles");
    group.bench_function("Mildly complex", |b| {
        b.iter(|| Cycle::from("[a b c {a b}%4 ! !]").unwrap())
    });
    group.bench_function("Quite complex", |b| {
        b.iter(|| Cycle::from("[0@2 <7 5>@2 3@2 5@1 0@1 3@2 <[7 2 3] [3 2 0]>@6]").unwrap())
    });
    group.bench_function("Pretty complex", |b| {
        b.iter(|| Cycle::from("<[{x@2 {x@3}}%4 x@3 x@4 ~!4] bd(3, 5) [[x@2 x@3 x@3]!2]>").unwrap())
    });
    group.bench_function("Very complex", |b| {
        b.iter(|| Cycle::from("<[{x@2 {x@3}}%4 [0@2 <7 5>@2 3@2 5@1 0@1 3@2 <[7 2 3] [3 2 0]>@6] x@4 ~!4] bd(3, 5) [[x@2 x@3 x@3]!2]>").unwrap())});
    group.finish();
}

pub fn nested_groups(c: &mut Criterion) {
    let mut group = c.benchmark_group("Nested Groups");
    group.bench_function("[[[]]]", |b| b.iter(|| Cycle::from("[[[]]]").unwrap()));
    group.bench_function("{{{}}}", |b| b.iter(|| Cycle::from("{{{}}}").unwrap()));
    group.bench_function("[{[]}]", |b| b.iter(|| Cycle::from("[{[]}]").unwrap()));
    group.bench_function("{[{}]}", |b| b.iter(|| Cycle::from("{[{}]}").unwrap()));
    group.finish();
}

// ---------------------------------------------------------------------------------------------

criterion_group! {
    name = cycle;
    config = Criterion::default();
    targets = parser, nested_groups
}

criterion_main!(cycle);
