//! Build regular expression from fluent API

use itertools::Itertools;
use std::fmt::{Display, Formatter, Result};

/// Generate Rust code
pub trait ToCode {
    fn to_code(&self) -> String;
}

/// A Regular Expression Pattern
#[derive(Debug)]
pub enum Pattern {
    /// Sequence of patterns
    Sequence(Vec<Pattern>),
    /// Text
    Text(String),
    /// Raw (unprocessed text), used internally
    Raw(String),
    /// Alternative
    Or(Vec<Pattern>),
    /// Repetition
    Many {
        exp: Box<Pattern>,
        low: u32,
        high: u32,
    },
    /// Digit
    Digit,
    /// Letter,
    Letter,
    /// Word characters
    WordCharacter,
    /// Start of line/input
    InputStart,
    /// End of line/input
    InputEnd,
    /// Negation
    Not(Box<Pattern>),
    /// Anything
    Any,
    /// Named group
    Named{exp:Box<Pattern>,name:String},
}

/// Generate regular expression from Pattern
impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Pattern::Sequence(v) => v
                .iter()
                .map(|e| match e {
                    Pattern::Or(..) if v.len() > 1 => write!(f, "({})", e),
                    _ => write!(f, "{}", e),
                })
                .collect(),
            Pattern::Text(t) => write!(f, "{}", t),
            Pattern::Raw(t) => write!(f, "{}", t),
            Pattern::Or(v) => v
                .iter()
                .intersperse(&Pattern::Raw("|".to_owned()))
                .map(|e| write!(f, "{}", e))
                .collect::<Result>(),
            Pattern::Many { exp, low, high } => {
                let mut s = format!("{}", exp);
                if s.len() > 2 || (s.len() == 2 && s.chars().into_iter().next().unwrap() != '\\') {
                    s = format!("({})", s);
                }
                match (low, high) {
                    (0, 1) => write!(f, "{}?", s),
                    (0, 0) => write!(f, "{}*", s),
                    (1, 0) => write!(f, "{}+", s),
                    (l, h) if l == h => write!(f, "{}{{{}}}", s, l),
                    (l, h) => write!(f, "{}{{{},{}}}", s, l, h),
                }
            }
            Pattern::Digit => write!(f, r"\d"),
            Pattern::Letter => write!(f, r"\pN"),
            Pattern::WordCharacter => write!(f, r"\w"),
            Pattern::InputStart => write!(f, "^"),
            Pattern::InputEnd => write!(f, "$"),
            Pattern::Not(exp)=> {
                match **exp {
                    Pattern::Digit => write!(f, r"\D"),
                    Pattern::Letter => write!(f, r"\PN"),
                    Pattern::WordCharacter => write!(f, r"\W"),
                    _ => write!(f, ""),
                }
            }
            Pattern::Any => write!(f,"."),
            Pattern::Named{exp, name} => write!(f, r"(?P<{}>{})",name,exp),
            
        }
    }
}

/// Convert a string into a text pattern
impl From<&str> for Pattern {
    fn from(s: &str) -> Pattern {
        Pattern::Text(s.to_owned())
    }
}

/// Convert a string into a text pattern
impl From<String> for Pattern {
    fn from(s: String) -> Pattern {
        Pattern::Text(s)
    }
}

/// Convert Pattern to Rust code
impl ToCode for Pattern {
    fn to_code(&self) -> String {
        self.to_inner_code(CodeState::root())
    }
}

/// Inner state for generation code
struct CodeState {
    /// are we at root of code?
    root: bool,
    /// are we first pattern in sequence?
    first: bool,
}

/// CodeState helper function
impl CodeState {
    /// Root, First
    fn root() -> Self {
        CodeState {
            root: true,
            first: true,
        }
    }

    /// First but not root
    fn first() -> Self {
        CodeState {
            root: false,
            first: true,
        }
    }

    /// Not first, not root
    fn next() -> Self {
        CodeState {
            root: false,
            first: false,
        }
    }
}

impl Pattern {
    /// Conversion to rust code
    fn to_inner_code(&self, state: CodeState) -> String {
        if state.first {
            match self {
                Pattern::Text(txt) => {
                    if state.root {
                        format!("text(\"{}\")", txt)
                    } else {
                        format!("\"{}\"", txt)
                    }
                }
                Pattern::Digit => "digit()".to_string(),
                Pattern::Any => "any()".to_string(),
                Pattern::Letter => "letter()".to_string(),
                Pattern::WordCharacter => "word_character()".to_string(),
                Pattern::Or(exps) => format!(
                    "either(({}))",
                    exps.iter()
                        .map(|e| e.to_inner_code(CodeState::first()))
                        .join(", ")
                ),
                Pattern::Not (exp ) => format!("any_except({})",exp.to_inner_code(CodeState::first())),
                Pattern::Many { exp, low, high } if low==high => format!(
                    "{}.times({})",
                    exp.to_inner_code(CodeState::first()),
                    low
                ),
                Pattern::Many { exp, low, high } => format!(
                    "{}.many({}, {})",
                    exp.to_inner_code(CodeState::first()),
                    low,
                    high
                ),
                Pattern::Named{exp,name}=>format!(r#"{}.named("{}")"#,exp.to_inner_code(CodeState::first()),name),
                Pattern::Sequence(exps) => {
                    let mut s = String::new();
                    for e in exps {
                        if s.is_empty() {
                            match e {
                                Pattern::InputStart => {
                                    s.push_str(&e.to_inner_code(CodeState::first()))
                                },
                                Pattern::Not(..) => {
                                    s.push_str(&e.to_inner_code(CodeState::first()))
                                },
                                Pattern::Digit => {
                                    s.push_str(&e.to_inner_code(CodeState::first()))
                                },
                                Pattern::Letter => {
                                    s.push_str(&e.to_inner_code(CodeState::first()))
                                },
                                Pattern::WordCharacter => {
                                    s.push_str(&e.to_inner_code(CodeState::first()))
                                },
                                _ => s.push_str(&format!(
                                    "start_with({})",
                                    e.to_inner_code(CodeState::first())
                                )),
                            };
                        } else {
                            s.push_str(&e.to_inner_code(CodeState::next()));
                        }
                    }
                    s
                }
                Pattern::InputStart => "at_start()".to_string(),
                _ => String::new(),
            }
        } else {
            match self {
                Pattern::Or(exps) => format!(
                    ".and_either(({}))",
                    exps.iter()
                        .map(|e| e.to_inner_code(CodeState::first()))
                        .join(", ")
                ),
                Pattern::Many { exp, low, high } => match (low, high) {
                    (0, 1) => format!(".and_maybe({})", exp.to_inner_code(CodeState::first())),
                    (0, 0) => format!(".and_maybe_many({})", exp.to_inner_code(CodeState::first())),
                    (1, 0) => format!(".and_many({})", exp.to_inner_code(CodeState::first())),
                    (l, h) if l == h => format!(
                        ".and_then({}).times({})",
                        exp.to_inner_code(CodeState::first()),
                        l
                    ),
                    _ => format!(
                        ".and_then({}).many({},{})",
                        exp.to_inner_code(CodeState::first()),
                        low,
                        high
                    ),
                },
                Pattern::InputEnd => ".must_end()".to_string(),
                Pattern::Named{exp,name}=>format!(r#".and_then({}.named("{}"))"#,exp.to_inner_code(CodeState::first()),name),
                _ => format!(".and_then({})", self.to_inner_code(CodeState::first())),
            }
        }
    }

    /// Chain an alternative
    pub fn and_either<PL: PatternList>(self, branches: PL) -> Self {
        self.push(Pattern::Or(branches.into_patterns().collect()))
    }

    /// Append a new pattern
    pub fn and_then<T: Into<Pattern>>(self, exp: T) -> Self {
        self.push(exp.into())
    }

    /// Append an optional pattern
    pub fn and_maybe<T: Into<Pattern>>(self, exp: T) -> Self {
        self.push(Pattern::Many {
            exp: Box::new(exp.into()),
            low: 0,
            high: 1,
        })
    }

    /// Append a pattern zero or more times
    pub fn and_maybe_many<T: Into<Pattern>>(self, exp: T) -> Self {
        self.push(Pattern::Many {
            exp: Box::new(exp.into()),
            low: 0,
            high: 0,
        })
    }

    /// Append a pattern one or more times
    pub fn and_many<T: Into<Pattern>>(self, exp: T) -> Self {
        self.push(Pattern::Many {
            exp: Box::new(exp.into()),
            low: 1,
            high: 0,
        })
    }

    /// Repeat the previous pattern between low and high times
    pub fn many(self, low: u32, high: u32) -> Self {
        match self {
            Pattern::Sequence(mut exps) if exps.len() > 0 => {
                let e = exps.pop().unwrap();
                exps.push(Pattern::Many {
                    exp: Box::new(e),
                    low: low,
                    high: high,
                });
                Pattern::Sequence(exps)
            }
            _ => Pattern::Many {
                exp: Box::new(self),
                low: low,
                high: high,
            },
        }
    }

    /// Repeat the previous pattern n times
    pub fn times(self, n: u32) -> Self {
        self.many(n, n)
    }

    /// name preceding pattern
    pub fn named<S: Into<String>>(self, name: S) -> Self {
        match self {
            Pattern::Sequence(mut exps) if exps.len() > 0 => {
                let e = exps.pop().unwrap();
                exps.push(Pattern::Named {
                    exp: Box::new(e),
                    name: name.into(),
                });
                Pattern::Sequence(exps)
            }
            _ => Pattern::Named {
                exp: Box::new(self),
                name: name.into(),
            },
        }
    }

    /// Must reach end of input
    pub fn must_end(self) -> Self {
        self.push(Pattern::InputEnd)
    }

    /*fn from_list(mut exprs: Vec<Pattern>) -> Pattern {
        if exprs.len()==1 {
            exprs.pop().unwrap()
        } else {
            Pattern::Sequence(exprs)
        }
    }*/

    /// Push a pattern next to self, maybe converting self into sequence
    fn push(self, p2: Pattern) -> Self {
        match self {
            Pattern::Sequence(mut exps) => {
                exps.push(p2);
                Pattern::Sequence(exps)
            }
            _ => Pattern::Sequence(vec![self, p2]),
        }
    }
}

/// Start of input
pub fn at_start() -> Pattern {
    Pattern::InputStart
}

/// Start matching with given pattern
pub fn start_with<T: Into<Pattern>>(exp: T) -> Pattern {
    exp.into()
}

/// Match given text
pub fn text(text: &str) -> Pattern {
    Pattern::Text(text.to_owned())
}

/// Match a digit
pub fn digit() -> Pattern {
    Pattern::Digit
}

/// Match a letter
pub fn letter() -> Pattern {
    Pattern::Letter
}

/// Match anything
pub fn anything() -> Pattern {
    Pattern::Any
}

/// Match a word character
pub fn word_character() -> Pattern {
    Pattern::WordCharacter
}

pub fn any_except<T: Into<Pattern>>(exp: T) -> Pattern {
    Pattern::Not(Box::new(exp.into()))
}

/// Match any of the given patterns
pub fn either<PL: PatternList>(branches: PL) -> Pattern {
    Pattern::Or(branches.into_patterns().collect())
}

/// Conversion into a list of patterns
pub trait PatternList {
    fn into_patterns(self) -> Box<dyn Iterator<Item = Pattern>>;
}

/// Convert a vector of patterns into an iterator
impl PatternList for Vec<Pattern> {
    fn into_patterns(self) -> Box<dyn Iterator<Item = Pattern>> {
        Box::new(self.into_iter())
    }
}

/// Convert a tuple of patterns into an iterator
impl<T1, T2> PatternList for (T1, T2)
where
    T1: Into<Pattern>,
    T2: Into<Pattern>,
{
    fn into_patterns(self) -> Box<dyn Iterator<Item = Pattern>> {
        Box::new(vec![self.0.into(), self.1.into()].into_iter())
    }
}

/// Convert a tripler of patterns into an iterator
impl<T1, T2, T3> PatternList for (T1, T2, T3)
where
    T1: Into<Pattern>,
    T2: Into<Pattern>,
    T3: Into<Pattern>,
{
    fn into_patterns(self) -> Box<dyn Iterator<Item = Pattern>> {
        Box::new(vec![self.0.into(), self.1.into(), self.2.into()].into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_build() {
        assert_eq!("Handel", text("Handel").to_string());
        assert_eq!("gray|grey", either(("gray", "grey")).to_string());
        assert_eq!(
            "gr(a|e)y",
            start_with("gr")
                .and_either(("a", "e"))
                .and_then("y")
                .to_string()
        );
        assert_eq!(
            "colou?r",
            start_with("colo").and_maybe("u").and_then("r").to_string()
        );
        assert_eq!(r"\d{2,3}", digit().many(2, 3).to_string());
        assert_eq!(
            r"^\d{4}-\d{2}-\d{2}$",
            at_start()
                .and_then(digit())
                .times(4)
                .and_then("-")
                .and_then(digit())
                .times(2)
                .and_then("-")
                .and_then(digit())
                .times(2)
                .must_end()
                .to_string()
        );
        assert_eq!(r#"\D\PN\W"#,any_except(digit()).and_then(any_except(letter())).and_then(any_except(word_character())).to_string());
        assert_eq!(
            r"(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})",
                start_with(digit()
                    .times(4)
                    .named("y"))
                .and_then("-")
                .and_then(digit()
                    .times(2)
                    .named("m"))
                .and_then("-")
                .and_then(digit()
                    .times(2)
                    .named("d"))
                .to_string()
        );
    }

    #[test]
    fn test_basic_tocode() {
        assert_eq!(r#"text("Handel")"#, text("Handel").to_code());
        assert_eq!(
            r#"either(("gray", "grey"))"#,
            either(("gray", "grey")).to_code()
        );
        assert_eq!(
            r#"start_with("gr").and_either(("a", "e")).and_then("y")"#,
            start_with("gr")
                .and_either(("a", "e"))
                .and_then("y")
                .to_code()
        );
        assert_eq!(
            r#"start_with("colo").and_maybe("u").and_then("r")"#,
            start_with("colo").and_maybe("u").and_then("r").to_code()
        );
        assert_eq!(r#"digit().many(2, 3)"#, digit().many(2, 3).to_code());
        assert_eq!(
            r#"at_start().and_then(digit()).times(4).and_then("-").and_then(digit()).times(2).and_then("-").and_then(digit()).times(2).must_end()"#,
            at_start()
                .and_then(digit().times(4))
                .and_then("-")
                .and_then(digit().times(2))
                .and_then("-")
                .and_then(digit().times(2))
                .must_end()
                .to_code()
        );
        assert_eq!(r#"any_except(digit()).and_then(any_except(letter())).and_then(any_except(word_character()))"#,any_except(digit()).and_then(any_except(letter())).and_then(any_except(word_character())).to_code());
        assert_eq!(r#"start_with(digit().times(4).named("y")).and_then("-").and_then(digit().times(2).named("m")).and_then("-").and_then(digit().times(2).named("d"))"#,start_with(digit().times(4).named("y")).and_then("-").and_then(digit().times(2).named("m")).and_then("-").and_then(digit().times(2).named("d")).to_code());
    }
}
