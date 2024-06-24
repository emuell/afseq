#[macro_use]
extern crate bencher;
use bencher::Bencher;

use afseq::tidal::Cycle;

// ---------------------------------------------------------------------------------------------

pub fn parser(bencher: &mut Bencher) {
    bencher.iter(|| Cycle::from("[a b c {a b}%4 ! !]").unwrap());
    bencher.iter(|| Cycle::from("[0@2 <7 5>@2 3@2 5@1 0@1 3@2 <[7 2 3] [3 2 0]>@6]").unwrap());
    bencher
        .iter(|| Cycle::from("<[{x@2 {x@3}}%4 x@3 x@4 ~!4] bd(3, 5) [[x@2 x@3 x@3]!2]>").unwrap());
    bencher
        .iter(|| Cycle::from("<[{x@2 {x@3}}%4 [0@2 <7 5>@2 3@2 5@1 0@1 3@2 <[7 2 3] [3 2 0]>@6] x@4 ~!4] bd(3, 5) [[x@2 x@3 x@3]!2]>").unwrap());
}

pub fn nested_groups(bencher: &mut Bencher) {
    bencher.iter(|| Cycle::from("[[[]]]").unwrap());
    bencher.iter(|| Cycle::from("{{{}}}").unwrap());
    bencher.iter(|| Cycle::from("[{[]}]").unwrap());
    bencher.iter(|| Cycle::from("{[{}]}").unwrap());
}

// ---------------------------------------------------------------------------------------------

benchmark_group!(cycle, parser, nested_groups);
benchmark_main!(cycle);
