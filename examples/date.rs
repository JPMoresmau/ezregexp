use ezregexp::{start_with, digit};
use regex::Regex;

fn main() {
    let p =start_with(digit().times(4).named("year"))
            .and_then("-")
            .and_then(digit().times(2).named("month"))
            .and_then("-")
            .and_then(digit().times(2).named("day"))
            .to_string();
    let re = Regex::new(&p.to_string()).unwrap();
    let caps = re.captures("2010-03-14").unwrap();

    assert_eq!("2010", &caps["year"]);
    assert_eq!("03", &caps["month"]);
    assert_eq!("14", &caps["day"]);
}
