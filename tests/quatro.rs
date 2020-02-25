#![feature(unboxed_closures)]
#![feature(fn_traits)]

use auto_curry::auto_curry;

#[auto_curry]
fn test(arg1: u8, arg2: i16, arg3: &'static str, arg4: ()) -> usize {
    format!("{}-{}-{}-{:?}", arg1, arg2, arg3, arg4).bytes().count()
}

fn main() {
    let first = test(1, 2, "3", ());
    let second = test(1, 2)("3", ());
    let third = test(1)(2)("3")(());
    let fourth = test(1)(2, "3")(());
    assert_eq!(first, second);
    assert_eq!(first, third);
    assert_eq!(first, fourth);
}