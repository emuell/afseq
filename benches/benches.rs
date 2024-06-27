use criterion::criterion_main;

// ---------------------------------------------------------------------------------------------

mod benchmarks;

// ---------------------------------------------------------------------------------------------

#[cfg(feature = "scripting")]
criterion_main!(
    benchmarks::scripted::scripted, //
    benchmarks::rhythm::rhythm,
    benchmarks::cycle::cycle,
);

#[cfg(not(feature = "scripting"))]
criterion_main!(
    benchmarks::rhythm::rhythm, //
    benchmarks::cycle::cycle,
);
