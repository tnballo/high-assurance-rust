// TODO: clap args --word-count and --lint
// TODO: update change log with hashes and word/diagram totals
fn main() {
    match md_analyze::Book::try_new(false) {
        Ok(book) => println!("\n{}", book),
        Err(e) => println!("Error: {:#?}", e),
    };
}
