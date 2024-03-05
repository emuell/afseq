use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

// --------------------------------------------------------------------------------------------------

#[derive(Parser)]
#[grammar = "tidal.pest"]
pub struct ExprParser;

// --------------------------------------------------------------------------------------------------

pub fn parse_mini(pair: Pair<Rule>, level: usize) {
    match pair.as_rule() {
        // container
        Rule::mini => {
            debug_assert!(level == 0);
            println!("mini: {}", pair.as_str());
            for item in pair.into_inner() {
                parse_mini(item, level + 1);
            }
        }

        // sequences
        Rule::polymeter => {
            println!("{} polymeter:", "\t".repeat(level));
            for item in pair.into_inner() {
                parse_mini(item, level + 1);
            }
        }
        Rule::sub_cycle => {
            println!("{} sub_cycle:", "\t".repeat(level));
            for item in pair.into_inner() {
                parse_mini(item, level + 1);
            }
        }

        Rule::stack_tail => {
            println!("{} stack_tail:", "\t".repeat(level));
            for item in pair.into_inner() {
                parse_mini(item, level + 1);
            }
        }
        Rule::choose_tail => {
            println!("{} choose_tail:", "\t".repeat(level));
            for item in pair.into_inner() {
                parse_mini(item, level + 1);
            }
        }
        Rule::dot_tail => {
            println!("{} dot_tail:", "\t".repeat(level));
            for item in pair.into_inner() {
                parse_mini(item, level + 1);
            }
        }

        // atomic expressions
        Rule::op_bjorklund => println!("{} bjorklund: {:?}", "\t".repeat(level), pair.as_str()),
        Rule::op_replicate => println!("{} op_replicate: {:?}", "\t".repeat(level), pair.as_str()),
        Rule::op_weight => println!("{} op_weight: {:?}", "\t".repeat(level), pair.as_str()),
        Rule::op_slow => println!("{} op_slow: {:?}", "\t".repeat(level), pair.as_str()),
        Rule::op_fast => println!("{} op_fast: {:?}", "\t".repeat(level), pair.as_str()),
        Rule::op_degrade => println!("{} op_degrade: {:?}", "\t".repeat(level), pair.as_str()),
        Rule::op_tail => println!("{} op_tail: {:?}", "\t".repeat(level), pair.as_str()),
        Rule::op_range => println!("{} op_range: {:?}", "\t".repeat(level), pair.as_str()),

        Rule::step => println!("{} step: {:?}", "\t".repeat(level), pair.as_str()),

        Rule::EOI => {}
        _ => unreachable!()
    }
}

pub fn parse(input: &str) {
    let mut parsed = ExprParser::parse(Rule::mini, input).unwrap();
    let level = 0;
    parse_mini(parsed.next().unwrap(), level);
}

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tidal() {
        parse("a b c");
        parse("[a b] c");
        parse("a|b c");
        parse("a _ b _");
        parse("a(2, 8) b c");
        parse("a {b c, d e, f} e");
    }
}
