// ANCHOR: stack_example
#[inline(never)]
fn recursive_count_down(x: usize) -> usize {
    if x == 0 {
        println!("Boom!");
        return x;
    } else {
        println!("{x}...");
    }

    let _ = recursive_count_down(x - 1);

    return x;
}

#[inline(never)]
fn square(x: usize) -> usize {
    x * x
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let x = args
        .iter()
        .nth(1)
        .expect("No arguments")
        .parse()
        .expect("Please provide a number");

    recursive_count_down(square(x));
}
// ANCHOR_END: stack_example
