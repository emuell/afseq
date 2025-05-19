use std::hint::black_box;

use criterion::{criterion_group, Criterion};

use afseq::tidal::Cycle;

// ---------------------------------------------------------------------------------------------

fn create_cycle() -> Cycle {
    // musical nonsense, trying to excessively use most of the supported features
    Cycle::from(
        r#"
[{g@2 {g@3}}%4 ! !],
[[a b c d](3,8,7)]
[{a b!2 c}%3],
[[0 1]!2 <a b>!4]
[[[{{}}]]],
[[1..12]:2]
[<[7? 2? 3?] [3 2 0]>@6 . [a,b,c,d,e,f]:5],
[<c4'maj g5'min d#'7#5 g8'5>]
"#,
    )
    .unwrap()
}

// ---------------------------------------------------------------------------------------------

pub fn parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("Cycle");
    group.bench_function("Parse", |b| b.iter(|| black_box(create_cycle())));
    group.finish();
}

pub fn generate(c: &mut Criterion) {
    let mut group = c.benchmark_group("Cycle");
    let mut cycle = create_cycle();
    group.bench_function("Generate", |b| b.iter(|| black_box(cycle.generate())));
    group.finish();
}

// ---------------------------------------------------------------------------------------------

criterion_group! {
    name = cycle;
    config = Criterion::default();
    targets = parse, generate
}
