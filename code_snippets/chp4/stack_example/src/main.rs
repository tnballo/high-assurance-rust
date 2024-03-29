// ANCHOR: stack_example
// ANCHOR: recursive_count_down
#[inline(never)]
fn recursive_count_down(x: usize) -> usize {
    // Base case
    if x == 0 {
        println!("Boom!");
        return x;
    // Recursive case
    } else {
        println!("{x}...");
        return recursive_count_down(x - 1);
    }
}
// ANCHOR_END: recursive_count_down

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

    let _ = recursive_count_down(square(x));
}
// ANCHOR_END: stack_example
