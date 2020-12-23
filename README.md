ezregexp
========
A Rust library to build regular expressions using a human-friendly fluent API.

### Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
ezregexp = "0.0.1"
```

and this to your crate root (if you're using Rust 2015):

```rust
extern crate ezregexp;
```

Here's a simple example that matches a date in YYYY-MM-DD format and capture the
year, month and day:

```rust
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

```

You can also use the library to generate the API calls from a regular expression:


```rust
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

```
