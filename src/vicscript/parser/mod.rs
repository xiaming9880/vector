extern crate pest;

use crate::{
    vicscript::{
        Assignment,
        query,
        query::{Literal, Path as QueryPath},
        Mapping,
        Result,
    },
    event::Value,
};

use pest::{
    Parser,
    iterators::{Pair, Pairs},
};

#[derive(Parser)]
#[grammar = "./vicscript/parser/grammar.pest"]
struct VicscriptParser;

fn path_from_pair(pair: Pair<Rule>) -> Result<String> {
   Ok(pair.as_str().get(1..).unwrap().to_string())
}

fn query_from_pair(pair: Pair<Rule>) -> Result<Box::<dyn query::Function>> {
    Ok(match pair.as_rule() {
        Rule::string => Box::new(Literal::from(Value::from(pair.into_inner().next().unwrap().as_str()))),
        Rule::null => Box::new(Literal::from(Value::Null)),
        Rule::number => Box::new(Literal::from(Value::from(pair.as_str().parse::<f64>().unwrap()))),
        Rule::boolean => {
            let v = if pair.as_str() == "true" {
                true
            } else {
                false
            };
            Box::new(Literal::from(Value::from(v)))
        },
        Rule::dot_path => Box::new(QueryPath::new(pair.as_str().get(1..).unwrap())),
        _ => unreachable!(),
    })
}

fn mapping_from_pairs(pairs: Pairs<Rule>) -> Result<Mapping> {
    let mut assignments = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::assignment => {
                let mut inner_rules = pair.into_inner();
                let path = path_from_pair(inner_rules.next().unwrap())?;
                let query = query_from_pair(inner_rules.next().unwrap())?;
                assignments.push(Assignment::new(path, query));
            },
            _ => (),
        }
    }
    Ok(Mapping::new(assignments))
}

pub fn parse(input: &str) -> Result<Mapping> {
    match VicscriptParser::parse(Rule::mapping, input) {
        Ok(a) => mapping_from_pairs(a),
        Err(err) => Err(format!("parse error{}", err)),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_parser_errors() {
        let cases = vec![
            (
                ".foo = {\"bar\"}",
                r###"parse error --> 1:8
  |
1 | .foo = {"bar"}
  |        ^---
  |
  = expected dot_path, boolean, null, string, or number"###,
            ),
            (
                ". = \"bar\"",
                r###"parse error --> 1:1
  |
1 | . = "bar"
  | ^---
  |
  = expected dot_path"###,
            ),
            (
                "foo = \"bar\"",
                r###"parse error --> 1:1
  |
1 | foo = "bar"
  | ^---
  |
  = expected dot_path"###,
            ),
            (
                ".foo.bar = \"baz\" and this",
                r###"parse error --> 1:18
  |
1 | .foo.bar = "baz" and this
  |                  ^---
  |
  = expected EOI"###,
            ),
        ];

        for (mapping, exp) in cases {
            assert_eq!(format!("{}", parse(mapping).err().unwrap()), exp);
        }
    }

    #[test]
    fn check_parser() {
        let cases = vec![
            (".foo = \"bar\"", Mapping::new(vec![
                Assignment::new("foo".to_string(), Box::new(Literal::from(Value::from("bar")))),
            ])),
            (".foo = true", Mapping::new(vec![
                Assignment::new("foo".to_string(), Box::new(Literal::from(Value::from(true)))),
            ])),
            (".foo = null", Mapping::new(vec![
                Assignment::new("foo".to_string(), Box::new(Literal::from(Value::Null))),
            ])),
            (".foo = 50.5", Mapping::new(vec![
                Assignment::new("foo".to_string(), Box::new(Literal::from(Value::from(50.5)))),
            ])),
            (".foo = .bar", Mapping::new(vec![
                Assignment::new("foo".to_string(), Box::new(QueryPath::new("bar"))),
            ])),
            (".foo = .bar\n.bar.buz = .qux.quz", Mapping::new(vec![
                Assignment::new("foo".to_string(), Box::new(QueryPath::new("bar"))),
                Assignment::new("bar.buz".to_string(), Box::new(QueryPath::new("qux.quz"))),
            ])),
        ];

        for (mapping, exp) in cases {
            assert_eq!(format!("{:?}", parse(mapping).ok().unwrap()), format!("{:?}", exp));
        }
    }
}