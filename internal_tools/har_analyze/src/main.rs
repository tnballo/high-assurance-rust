use clap::{
    builder::{styling::AnsiColor, Styles},
    Parser,
};
use color_eyre::eyre::Result;
use colored::*;
use lazy_static::lazy_static;

lazy_static! {
    static ref CMD_COLOR: Styles = Styles::styled()
        .header(AnsiColor::Yellow.on_default())
        .usage(AnsiColor::Green.on_default())
        .valid(AnsiColor::Green.on_default())
        .error(AnsiColor::Red.on_default())
        .invalid(AnsiColor::Red.on_default())
        .literal(AnsiColor::Cyan.on_default())
        .placeholder(AnsiColor::BrightBlue.on_default());
}

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    term_width = 150,
    styles = CMD_COLOR.clone(),
)]
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

    /// Update page/diagram count badges and missing meta tags.
    #[arg(short, long)]
    update: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let book = har_analyze::Book::try_new(args.lint).unwrap();

    // Status Report
    if args.metrics {
        println!("\n{book}");
    }

    // Update/fix
    if args.update {
        har_analyze::update_badges(&book).unwrap();
        har_analyze::update_meta_tags(&book).unwrap();
        println!("Updates {}", "OK".green());
    }

    // Verify
    if args.lint {
        book.get_non_chp_linter().run(args.log_warn).unwrap();
        book.get_chp_intro_linter().run(args.log_warn).unwrap();
        book.get_chp_sections_linter().run(args.log_warn).unwrap();
        book.get_svg_linter().run(args.log_warn).unwrap();
        println!("Lint {}", "OK".green());
    }

    Ok(())
}
