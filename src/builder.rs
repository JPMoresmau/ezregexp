use std::fmt::{Display,Formatter, Result};
use itertools::Itertools;

pub trait ToCode {
    fn to_code(&self) -> String;
}

pub struct Pattern {
    exps: Vec<Expression>,
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.exps.iter().map(|e| {
            match e {
                Expression::Or(..) if self.exps.len()>1 => write!(f, "({})", e),
                _=> write!(f, "{}", e),
            }
        }).collect()
    }
}

impl From<&str> for Pattern {
    fn from(s:&str) -> Pattern {
        Pattern{exps:vec![Expression::Text(s.to_owned())]}
    }
}

impl From<String> for Pattern {
    fn from(s:String) -> Pattern {
        Pattern{exps:vec![Expression::Text(s)]}
    }
}

impl ToCode for Pattern {
    fn to_code(&self) -> String{
        if self.exps.len()==1 {
            match &self.exps[0]{
                Expression::Text(txt) => format!("text(\"{}\")",txt),
                Expression::Digit => "digit()".to_string(),
                Expression::Or(exps) => format!("either(({}))",exps.iter().map(|e| e.to_code()).join(", ")),
                Expression::Many{exp,low,high} => format!("{}.many({}, {})",exp.to_code(),low,high),
                _ => String::new(),
            }
        } else if self.exps.len()>1 {
            let mut s=String::new();
            for e in &self.exps {
                if s.is_empty(){
                    s.push_str(&format!("start_with({})",e.to_code()));
                } else {
                    s.push_str(& match e {
                        Expression::Or(exps) => format!(".and_either(({}))",exps.iter().map(|e| e.to_code()).join(", ")),
                        Expression::Many{exp,low,high} => {
                            match (low,high){
                                (0,1)=>format!(".and_maybe({})",exp.to_code()),
                                (0,0)=>format!(".and_maybe_many({})",exp.to_code()),
                                (1,0)=>format!(".and_many({})",exp.to_code()),
                                _=>format!(".and_then({}).many({},{})",exp.to_code(),low,high),

                            }
                        }
                        _ =>  format!(".and_then({})",e.to_code()),
                    });
                }
            }
            s
        } else {
            String::new()
        }
    }
}

impl Pattern {

    pub fn and_either<PL:PatternList>(&mut self, branches: PL)-> &mut Self {
        self.exps.push(Expression::Or(branches.into_patterns().map(|p| Expression::from_list(p.exps)).collect()));
        self
    }

    pub fn and_then<T:Into<Expression>>(&mut self, exp:T) -> &mut Self {
        self.exps.push(exp.into());
        self
    }

    pub fn and_maybe<T:Into<Expression>>(&mut self, exp:T) -> &mut Self {
        self.exps.push(Expression::Many{exp:Box::new(exp.into()),low:0,high:1});
        self
    }

    pub fn and_maybe_many<T:Into<Expression>>(&mut self, exp:T) -> &mut Self {
        self.exps.push(Expression::Many{exp:Box::new(exp.into()),low:0,high:0});
        self
    }

    pub fn and_many<T:Into<Expression>>(&mut self, exp:T) -> &mut Self {
        self.exps.push(Expression::Many{exp:Box::new(exp.into()),low:1,high:0});
        self
    }

    pub fn many(&mut self, low: usize, high: usize) -> &mut Self {
        if let Some(e) = self.exps.pop(){
            self.exps.push(Expression::Many{exp:Box::new(e),low:low,high:high});
        }
        self
    }
}


#[derive(Debug)]
pub enum Expression {
    Sequence(Vec<Expression>),
    Text(String),
    Raw(String),
    Or(Vec<Expression>),
    Many{exp:Box<Expression>,
         low: usize,
         high: usize,
        },
    Digit,
}

impl From<&str> for Expression {
    fn from(s:&str) -> Expression {
        Expression::Text(s.to_owned())
    }
}

impl From<String> for Expression {
    fn from(s:String) -> Expression {
        Expression::Text(s)
    }
}

impl From<Pattern> for Expression {
    fn from(p:Pattern) -> Expression {
        Expression::Sequence(p.exps)
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Expression::Sequence(v)=>v.iter().map(|e| {
                match e {
                    Expression::Or(..) if v.len()>1=> write!(f, "({})", e),
                    _=> write!(f, "{}", e),
                }
            }).collect(),
            Expression::Text(t)=> write!(f, "{}", t),
            Expression::Raw(t)=> write!(f, "{}", t),
            Expression::Or(v)=> {
                v.iter().intersperse(&Expression::Raw("|".to_owned())).map(|e|{
                    write!(f, "{}", e)
                }).collect::<Result>()
            },
            Expression::Many{exp,low,high}=>{
                let mut s = format!("{}",exp);
                if s.len()>2 || (s.len()==2 && s.chars().into_iter().next().unwrap() !='\\') {
                    s=format!("({})",s);
                }
                match (low,high){
                    (0,1)=>write!(f,"{}?", s),
                    (0,0)=>write!(f,"{}*", s),
                    (1,0)=>write!(f,"{}+", s),
                    (l,h)=>write!(f,"{}{{{},{}}}", s, l, h),
                }
                
            },
            Expression::Digit=>write!(f, r"\d"),
        }
    }
}

impl ToCode for Expression {
    fn to_code(&self) -> String {
        //println!("{:?}",self);
        match self {
            Expression::Text(txt) => format!("\"{}\"", txt),
            Expression::Digit => "digit()".to_string(),
            _ => String::new(),
        }
    }
}

impl Expression {
    pub fn from_list(mut exprs: Vec<Expression>) -> Expression {
        if exprs.len()==1 {
            exprs.pop().unwrap()
        } else {
            Expression::Sequence(exprs)
        }
    }
}


pub fn start_with<T:Into<Expression>>(exp:T)-> Pattern {
    let mut p = Pattern{exps:vec![]};
    p.exps.push(exp.into());
    p
}

pub fn text(text: &str)-> Pattern {
    let mut p = Pattern{exps:vec![]};
    p.exps.push(Expression::Text(text.to_owned()));
    p
}

pub fn digit() -> Pattern {
    let mut p = Pattern{exps:vec![]};
    p.exps.push(Expression::Digit);
    p
}

pub fn either<PL:PatternList>(branches: PL)-> Pattern {
    let mut p = Pattern{exps:vec![]};
    p.exps.push(Expression::Or(branches.into_patterns().map(|p| Expression::from_list(p.exps)).collect()));
    p
}


pub trait PatternList{
    fn into_patterns(self) -> Box<dyn Iterator<Item=Pattern>>;
}

impl PatternList for Vec<Pattern> {
    fn into_patterns(self)-> Box<dyn Iterator<Item=Pattern>>{
        Box::new(self.into_iter())
    }
}

impl<T1,T2> PatternList for (T1,T2) where T1:Into<Pattern>,T2:Into<Pattern>, {
    fn into_patterns(self)-> Box<dyn Iterator<Item=Pattern>>{
        Box::new(vec![self.0.into(),self.1.into()].into_iter())
    }
}

impl<T1,T2,T3> PatternList for (T1,T2,T3) where T1:Into<Pattern>,T2:Into<Pattern>,T3:Into<Pattern>,{
    fn into_patterns(self)-> Box<dyn Iterator<Item=Pattern>>{
        Box::new(vec![self.0.into(),self.1.into(), self.2.into()].into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_build(){
        assert_eq!("Handel",text("Handel").to_string());
        assert_eq!("gray|grey",either(("gray","grey")).to_string());
        assert_eq!("gr(a|e)y",start_with("gr").and_either(("a","e")).and_then("y").to_string());
        assert_eq!("colou?r",start_with("colo").and_maybe("u").and_then("r").to_string());
        assert_eq!(r"\d{2,3}",digit().many(2, 3).to_string());
    }

    #[test]
    fn test_basic_tocode(){
        assert_eq!(r#"text("Handel")"#,text("Handel").to_code());
        assert_eq!(r#"either(("gray", "grey"))"#,either(("gray","grey")).to_code());
        assert_eq!(r#"start_with("gr").and_either(("a", "e")).and_then("y")"#,start_with("gr").and_either(("a","e")).and_then("y").to_code());
        assert_eq!(r#"start_with("colo").and_maybe("u").and_then("r")"#,start_with("colo").and_maybe("u").and_then("r").to_code());
        assert_eq!(r#"digit().many(2, 3)"#,digit().many(2, 3).to_code());
    }
}

