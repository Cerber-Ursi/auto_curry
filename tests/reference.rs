#![feature(unboxed_closures)]
#![feature(fn_traits)]

use auto_curry::auto_curry;

struct Ref<'a, T>(&'a T);

#[auto_curry]
fn test(_: &str, _: Ref<'_, str>, _: usize) {}

fn main() {
    let composite = String::from("part1 part2");
    let parts = composite.split_ascii_whitespace();
    let (arg1, arg2) = (parts.next(), parts.next());
    let curried = test(arg1, Ref(arg2));
    curried(0);
}