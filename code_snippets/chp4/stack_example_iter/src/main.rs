// ANCHOR: iterative_count_down
#[inline(never)]
fn iterative_count_down(x: usize) {
    for i in (0..=x).rev() {
        match i {
            i if i == 0 => println!("Boom!"),
            _ => println!("{i}..."),
        }
    }
}
// ANCHOR_END: iterative_count_down

#[inline(never)]
fn square(x: usize) -> usize {
    x * x
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // 1st arg is binary name, e.g. "./stack_example 2"
    assert!(args.len() <= 2, "Too many arguments - enter one number");

    let x = args
        .iter()
        .nth(1)
        .expect("No arguments")
        .parse()
        .expect("Please provide a number");

    iterative_count_down(square(x));
}
