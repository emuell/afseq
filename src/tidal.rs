use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

// --------------------------------------------------------------------------------------------------

#[derive(Parser)]
#[grammar = "tidal.pest"]
pub struct ExprParser;

// --------------------------------------------------------------------------------------------------

pub fn parse_mini(pair: Pair<Rule>) {
    match pair.as_rule() {
        Rule::mini => {
            parse_mini(pair.into_inner().next().unwrap());
        }
        Rule::sequence => {
            let mut _inner = pair.into_inner();
            _inner
                .map(|pair| {
                    let _sequence_value = pair.as_rule();
                    println!("sequence value: {:?}", _sequence_value)
                })
                .last()
                .unwrap()
        }
        Rule::step => {
            println!("step: {:?}", pair.as_str());
        }
        Rule::stack_or_choose => {
            let mut _inner = pair.into_inner();
            _inner
                .map(|pair| {
                    let _sequence = pair.as_rule();
                    parse_mini(pair.into_inner().next().unwrap());
                })
                .last()
                .unwrap()
        }
        Rule::ws => {}
        _ => println!("unhandled: {:?}", pair.as_rule()),
    }
}

pub fn parse(input: &str) {
    let mut parsed = ExprParser::parse(Rule::mini, input).unwrap();
    parse_mini(parsed.next().unwrap())
}

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tidal() {
        parse("1 2 3");
        parse("[1 2] 3");
        parse("1|2 3");
        parse("1 _ 2");
    }
}
