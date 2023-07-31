fn get_greeting() -> String {
    String::from("Hello")
}

fn main() {
    let mut greeting = get_greeting();
    greeting.push('!');
    println!("{}", greeting);
}
