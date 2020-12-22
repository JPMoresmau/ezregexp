use crate::builder::Pattern;
use regex_syntax::ast::{
    parse::Parser, Alternation, Assertion, AssertionKind, Ast, Class, ClassPerl, ClassPerlKind,
    Concat, Error, Group, Literal, Repetition, RepetitionKind, RepetitionOp, RepetitionRange,
};

pub fn explain(regex: &str) -> Result<Pattern, Error> {
    let mut p = Parser::new();
    p.parse(regex).and_then(|a| {
        println!("ast: {:?}", a);
        do_explain(&a)
    })
}

fn do_explain(ast: &Ast) -> Result<Pattern, Error> {
    match ast {
        Ast::Concat(Concat { asts, .. }) => Ok(simplify(
            asts.iter()
                .map(|a| do_explain(a))
                .collect::<Result<Vec<Pattern>, Error>>()?,
        )),
        Ast::Literal(Literal { c, .. }) => Ok(Pattern::Text(format!("{}", c))),
        Ast::Alternation(Alternation { asts, .. }) => Ok(Pattern::Or(
            asts.iter()
                .map(|a| do_explain(a))
                .collect::<Result<Vec<Pattern>, Error>>()?,
        )),
        Ast::Group(Group { ast, .. }) => do_explain(ast),
        Ast::Repetition(Repetition { ast, op, .. }) => {
            let bds = bounds(op);
            Ok(Pattern::Many {
                exp: Box::new(do_explain(ast)?),
                low: bds.0,
                high: bds.1,
            })
        }
        Ast::Class(Class::Perl(ClassPerl {
            kind: ClassPerlKind::Digit,
            ..
        })) => Ok(Pattern::Digit),
        Ast::Assertion(Assertion {
            kind: AssertionKind::StartLine,
            ..
        }) => Ok(Pattern::InputStart),
        Ast::Assertion(Assertion {
            kind: AssertionKind::EndLine,
            ..
        }) => Ok(Pattern::InputEnd),
        _ => Ok(Pattern::Raw(String::new())),
    }
}

fn bounds(op: &RepetitionOp) -> (u32, u32) {
    match &op.kind {
        RepetitionKind::ZeroOrOne => (0, 1),
        RepetitionKind::ZeroOrMore => (0, 0),
        RepetitionKind::OneOrMore => (1, 0),
        RepetitionKind::Range(r) => match r {
            RepetitionRange::AtLeast(m) => (*m, 0),
            RepetitionRange::Exactly(m) => (*m, *m),
            RepetitionRange::Bounded(l, h) => (*l, *h),
        },
    }
}

fn simplify(exps: Vec<Pattern>) -> Pattern {
    let mut nexps = vec![];
    for p in exps.into_iter() {
        if let Pattern::Text(t) = p {
            let op0 = nexps.pop();
            if let Some(Pattern::Text(mut t0)) = op0 {
                t0.push_str(&t);
                nexps.push(Pattern::Text(t0));
            } else {
                if let Some(p0) = op0 {
                    nexps.push(p0);
                }
                nexps.push(Pattern::Text(t));
            }
        } else {
            nexps.push(p);
        }
    }
    if nexps.len() == 1 {
        nexps.pop().unwrap()
    } else {
        Pattern::Sequence(nexps)
    }
}

/*
struct ExplainState {
    stack:Vec<State>,
}

enum State {
    Root,
    String,
    Expression,
}

impl Default for ExplainState {
    fn default() -> Self {
        ExplainState{stack:vec![State::Root]}
    }


}

impl ExplainState {
    fn string(&mut self) -> bool {
        if let Some(State::String) = self.stack.last(){
            return false;
        } else {
            self.stack.push(State::String);
        }
        true
    }

    fn unstring(&mut self) -> bool {
        if let Some(State::String) = self.stack.last(){
            self.stack.pop();
            return true;
        }
        false
    }

    fn is_root(&self) -> bool {
        if let Some(State::Root) = self.stack.last(){
            return true;
        }
        false
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::ToCode;

    #[test]
    fn test_basic_explain() {
        assert_eq!(
            Ok(r#"text("Handel")"#.to_owned()),
            explain("Handel").map(|p| p.to_code())
        );
        assert_eq!(
            Ok(r#"either(("gray", "grey"))"#.to_owned()),
            explain("gray|grey").map(|p| p.to_code())
        );
        assert_eq!(
            Ok(r#"start_with("gr").and_either(("a", "e")).and_then("y")"#.to_owned()),
            explain("gr(a|e)y").map(|p| p.to_code())
        );
        assert_eq!(
            Ok(r#"start_with("colo").and_maybe("u").and_then("r")"#.to_owned()),
            explain("colou?r").map(|p| p.to_code())
        );
        assert_eq!(
            Ok(r#"digit().many(2, 3)"#.to_owned()),
            explain(r#"\d{2,3}"#).map(|p| p.to_code())
        );
        assert_eq!(Ok(r#"at_start().and_then(digit()).times(4).and_then("-").and_then(digit()).times(2).and_then("-").and_then(digit()).times(2).must_end()"#.to_owned()),explain(r"^\d{4}-\d{2}-\d{2}$").map(|p| p.to_code()));
    }
}
