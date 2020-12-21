use regex_syntax::ast::{Alternation, Ast, Concat, Error, Group, Literal, parse::Parser};
use itertools::Itertools;

pub fn explain(regex: &str) -> Result<String,Error> {
    let mut p=Parser::new();
    p.parse(regex).and_then(|a| { println!("ast: {:?}",a); do_explain(&a, &mut ExplainState::default())})

}

fn do_explain(ast: &Ast, state: &mut ExplainState) -> Result<String,Error> {
   
    let mut s=String::new();
    match ast {
        Ast::Concat(Concat{asts,..})=>{
            if state.is_root(){
                s.push_str("start_with(");
            }
            let mut state2=ExplainState::default();
            state2.stack.push(State::Expression);
            let vs:Vec<String> = asts.iter().map(|a| do_explain(a, &mut state2)).collect::<Result<Vec<String>,Error>>()?;
            vs.into_iter().for_each(|s2|s.push_str(&s2));
            if state2.unstring() {
                s.push('"');
            }
            if state.is_root(){
                s.push(')');
            }
        },
        Ast::Literal(Literal{c,..}) => {
            if state.string(){
                s.push('"');
            }
            s.push(*c);
        },
        Ast::Alternation(Alternation{asts,..})=>{
            let mut state2=ExplainState::default();
            state2.stack.push(State::Expression);
            if state.is_root(){
                s.push_str("either((");
            } else {
                s.push_str(".and_either((");
            }
            let vs:Vec<String> = asts.iter().map(|a| {
                let r=do_explain(a, &mut state2);
                if state2.unstring() {
                    r.map(|mut s| { s.push('"'); s})
                } else {
                    r
                }
                })
                .intersperse(Ok(",".to_string()))
                .collect::<Result<Vec<String>,Error>>()?;
            vs.into_iter().for_each(|s2|s.push_str(&s2));
            s.push_str("))");
        },
        Ast::Group(Group{ast,..})=>{
            if state.unstring() {
                s.push('"');
            }
            s.push(')');
            let mut state2=ExplainState::default();
            state2.stack.push(State::Expression);
            let s2=do_explain(ast, &mut state2)?;
            if state2.unstring() {
                s.push('"');
            }
            s.push_str(&s2);
        }
        _ => {},
    }
    Ok(s)
}


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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_explain(){
        assert_eq!(Ok(r#"start_with("Handel")"#.to_owned()),explain("Handel"));
        assert_eq!(Ok(r#"either(("gray","grey"))"#.to_owned()),explain("gray|grey"));
        assert_eq!(Ok(r#"start_with("gr").and_either(("a","e")).and_then("y")"#.to_owned()),explain("gr(a|e)y"));
    }
}