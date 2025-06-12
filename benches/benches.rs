use criterion::criterion_main;

// ---------------------------------------------------------------------------------------------

mod benchmarks;

// ---------------------------------------------------------------------------------------------

#[cfg(feature = "scripting")]
criterion_main!(
    benchmarks::scripted::scripted, //
    benchmarks::pattern::pattern,
    benchmarks::cycle::cycle,
);

#[cfg(not(feature = "scripting"))]
criterion_main!(
    benchmarks::pattern::pattern, //
    benchmarks::cycle::cycle,
);
