use clap::Parser;
use color_eyre::eyre::Result;
use colored::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[group(required = true, multiple = true)]
struct Args {
    /// Print page/diagram count metrics.
    #[arg(short, long)]
    metrics: bool,

    /// Run custom linter.
    #[arg(short, long)]
    lint: bool,

    /// Log linter warnings. If false (default), warnings become hard errors.
    #[arg(long, requires = "lint")]
    log_warn: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let book = har_analyze::Book::try_new(args.lint).unwrap();

    if args.metrics {
        println!("\n{}", book);
    }

    if args.lint {
        book.get_non_chp_linter().run(args.log_warn).unwrap();
        book.get_chp_intro_linter().run(args.log_warn).unwrap();
        book.get_chp_sections_linter().run(args.log_warn).unwrap();
        book.get_svg_linter().run(args.log_warn).unwrap();
        println!("Lint {}", "OK".green());
    }

    Ok(())
}
