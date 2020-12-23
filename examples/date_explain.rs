use ezregexp::{ToCode,explain};
use regex::Regex;

fn main() {
    let s = r"(?x)
    (?P<year>\d{4})  # the year
    -
    (?P<month>\d{2}) # the month
    -
    (?P<day>\d{2})   # the day
    ";
    let re = Regex::new(s).unwrap();
    let caps = re.captures("2010-03-14").unwrap();

    assert_eq!("2010", &caps["year"]);
    assert_eq!("03", &caps["month"]);
    assert_eq!("14", &caps["day"]);

    println!("{}",explain(s).unwrap().to_code());
}
