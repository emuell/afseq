use std::collections::HashMap;

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use crate::types::{Function, Kind, LuaKind, Var};

#[derive(Parser)]
#[grammar = "./lua_parser.pest"]
pub struct LuaParser {}
impl LuaParser {
    /// parse a string into a type definition of Kind
    pub fn type_def(input: &str) -> Kind {
        match Self::parse(Rule::type_def, input) {
            Ok(mut pairs) => {
                let next = pairs.next().unwrap();
                let pair = next.into_inner().next().unwrap();
                Self::kind(pair)
            }
            Err(err) => {
                // warn about parse errors
                println!("\x1b[33m{}\x1b[0m", err);
                Kind::Unresolved(input.to_string())
            }
        }
    }

    fn as_string(pair: &Pair<Rule>) -> String {
        pair.as_span().as_str().to_string()
    }

    fn named_var(pair: Pair<Rule>) -> Var {
        let mut inner = pair.into_inner();
        let name = Some(Self::as_string(&inner.next().unwrap()));
        let next = inner.next().unwrap();
        match next.as_rule() {
            Rule::nullable_tail => Var {
                file: None,
                line_number: None,
                name,
                kind: Kind::Nullable(Box::new(Self::kind(inner.next().unwrap()))),
                desc: None,
            },
            _ => Var {
                file: None,
                line_number: None,
                name,
                kind: Self::kind(next),
                desc: None,
            },
        }
    }

    fn returns(pair: Option<Pair<Rule>>) -> Vec<Var> {
        let mut returns: Vec<Var> = vec![];
        if let Some(pair) = pair {
            for p in pair.into_inner() {
                returns.push(match p.as_rule() {
                    Rule::named_var => Self::named_var(p),
                    Rule::enumeration | Rule::complex_type => Var {
                        file: None,
                        line_number: None,
                        kind: Self::kind(p),
                        name: None,
                        desc: None,
                    },
                    _ => unreachable!(),
                })
            }
        }
        returns
    }

    fn args(pair: Pair<Rule>) -> Vec<Var> {
        let mut params: Vec<Var> = vec![];
        for arg in pair.into_inner() {
            match arg.as_rule() {
                Rule::named_var => params.push(Self::named_var(arg)),
                Rule::valid_name => params.push(Var {
                    file: None,
                    line_number: None,
                    name: Some(Self::as_string(&arg)),
                    kind: Kind::Lua(LuaKind::Any),
                    desc: None,
                }),
                Rule::nullable_tail => {
                    if let Some(last) = params.last_mut() {
                        last.kind = Kind::Nullable(Box::new(last.kind.clone()))
                    }
                }
                Rule::vararg => params.push(Var {
                    file: None,
                    line_number: None,
                    name: None,
                    kind: Kind::Variadic(Box::new(Self::kind(arg.into_inner().next().unwrap()))),
                    desc: None,
                }),
                _ => unreachable!(),
            };
        }
        params
    }

    fn kind(pair: Pair<Rule>) -> Kind {
        let s = Self::as_string(&pair);
        match pair.as_rule() {
            Rule::id => Kind::Unresolved(s),
            Rule::string_literal => Kind::Literal(
                Box::new(LuaKind::String),
                Self::as_string(&pair.into_inner().next().unwrap()),
            ),
            Rule::number_literal => Kind::Literal(Box::new(LuaKind::Integer), s),
            Rule::lua_type => {
                match serde_json::from_str::<LuaKind>(
                    format!("\"{}\"", pair.as_span().as_str()).as_str(),
                ) {
                    Ok(lk) => Kind::Lua(lk),
                    Err(_) => Kind::Unresolved(s),
                }
            }
            Rule::complex_type => {
                let mut inner = pair.into_inner();
                let mut k = Self::kind(inner.next().unwrap());
                for tail in inner {
                    match tail.as_rule() {
                        Rule::array_tail => k = Kind::Array(Box::new(k)),
                        Rule::nullable_tail => k = Kind::Nullable(Box::new(k)),
                        _ => (),
                    }
                }
                k
            }
            Rule::enumeration => Kind::Enum(pair.into_inner().map(|p| Self::kind(p)).collect()),
            Rule::table_type => {
                let mut inner = pair.into_inner();
                let key = inner.next().unwrap();
                let value = inner.next().unwrap();
                Kind::Table(Box::new(Self::kind(key)), Box::new(Self::kind(value)))
            }
            Rule::object => {
                let inner = pair.into_inner();
                let mut fields = HashMap::new();
                for f in inner {
                    let mut fi = f.into_inner();
                    let key = fi.next().unwrap();
                    if let Some(t) = fi.next() {
                        fields.insert(
                            key.as_span().as_str().to_string(),
                            Box::new(Self::kind(t.clone())),
                        );
                    }
                }
                Kind::Object(fields)
            }
            Rule::function => {
                let mut inner = pair.into_inner();
                let file = None;
                let line_number = None;
                let name = Some(Self::as_string(&inner.next().unwrap()));
                let params = Self::args(inner.next().unwrap());
                let returns = Self::returns(inner.next());
                Kind::Function(Function {
                    file,
                    line_number,
                    name,
                    params,
                    returns,
                    desc: None,
                })
            }
            Rule::fun => {
                let mut inner = pair.into_inner();
                let file = None;
                let line_number = None;
                let params = Self::args(inner.next().unwrap());
                let returns = Self::returns(inner.next());
                Kind::Function(Function {
                    file,
                    line_number,
                    params,
                    returns,
                    name: None,
                    desc: None,
                })
            }
            Rule::method => {
                let mut inner = pair.into_inner();
                let _parent = Self::as_string(&inner.next().unwrap());
                let file = None;
                let line_number = None;
                let name = Some(Self::as_string(&inner.next().unwrap()));
                let params = Self::args(inner.next().unwrap());
                let returns = Self::returns(inner.next());
                Kind::Function(Function {
                    file,
                    line_number,
                    name,
                    params,
                    returns,
                    desc: None,
                })
            }
            _ => {
                println!("{:?}", pair.as_rule());
                unreachable!()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_type(input: &str, k: Kind) -> Result<(), String> {
        assert_eq!(LuaParser::type_def(input), k);
        Ok(())
    }

    fn var<N: Into<Option<String>>>(name: N, kind: Kind) -> Var {
        let name = name.into();
        Var {
            file: None,
            line_number: None,
            name,
            kind,
            desc: None,
        }
    }

    fn string_literal(s: &str) -> Kind {
        Kind::Literal(Box::new(LuaKind::String), s.to_string())
    }

    #[test]
    pub fn parse() -> Result<(), String> {
        assert_type("integer", Kind::Lua(LuaKind::Integer))?;
        assert_type(
            "integer[]",
            Kind::Array(Box::new(Kind::Lua(LuaKind::Integer))),
        )?;
        assert_type("\"a\"", string_literal("a"))?;
        assert_type(
            "\"a\"|\"b\"",
            Kind::Enum(vec![string_literal("a"), string_literal("b")]),
        )?;
        assert_type(
            "(\"a\"?|\"b\"[])",
            Kind::Enum(vec![
                Kind::Nullable(Box::new(string_literal("a"))),
                Kind::Array(Box::new(string_literal("b"))),
            ]),
        )?;
        assert_type(
            "(\"a\"?|\"b\")?",
            Kind::Nullable(Box::new(Kind::Enum(vec![
                Kind::Nullable(Box::new(string_literal("a"))),
                string_literal("b"),
            ]))),
        )?;
        assert_type(
            "integer[][]",
            Kind::Array(Box::new(Kind::Array(Box::new(Kind::Lua(LuaKind::Integer))))),
        )?;
        assert_type(
            "function a.b.c(i: integer, ...string)",
            Kind::Function(Function {
                file: None,
                line_number: None,
                name: Some(String::from("a.b.c")),
                params: vec![
                    var("i".to_string(), Kind::Lua(LuaKind::Integer)),
                    var(None, Kind::Variadic(Box::new(Kind::Lua(LuaKind::String)))),
                ],
                returns: vec![],
                desc: None,
            }),
        )?;
        assert_type(
            "(method) renoise.Song:test(a: integer, b:integer|string?)",
            Kind::Function(Function {
                file: None,
                line_number: None,
                name: Some(String::from("test")),
                params: vec![
                    var("a".to_string(), Kind::Lua(LuaKind::Integer)),
                    var(
                        "b".to_string(),
                        Kind::Enum(vec![
                            Kind::Lua(LuaKind::Integer),
                            Kind::Nullable(Box::new(Kind::Lua(LuaKind::String))),
                        ]),
                    ),
                ],
                returns: vec![],
                desc: None,
            }),
        )?;
        Ok(())
    }
}
