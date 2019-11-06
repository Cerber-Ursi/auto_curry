#![feature(unboxed_closures)]
#![feature(fn_traits)]

use auto_curry::auto_curry;

#[auto_curry]
fn test(arg1: u8, arg2: &str) -> String {
    format!("{}-{}", arg1, arg2);
}

fn main() {
    assert_eq!(test(0, "test"), test(0)("test"));
}
