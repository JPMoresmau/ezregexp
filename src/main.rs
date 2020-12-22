use ezregexp::builder::{at_start, digit};

fn main() {
    println!("{}", at_start()
        .and_then(digit())
        .times(4)
        .and_then("-")
        .and_then(digit())
        .times(2)
        .and_then("-")
        .and_then(digit())
        .times(2)
        .must_end());
}
