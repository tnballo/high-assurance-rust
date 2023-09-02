// TODO: clap args --word-count and --lint
// TODO: update change log with hashes and word/diagram totals
fn main() {
    let book = md_analyze::Book::try_new(true).unwrap();

    println!("\n{}", book);

    book.get_all_section_linter().run().unwrap();
}
