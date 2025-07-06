use std::{env, ffi::OsStr, path::Path, process};
use supplychain_policy::Policy;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Error: provide exactly one argument - the Cargo.toml filepath");
        process::exit(1);
    }

    if args[1].eq_ignore_ascii_case("help")
        || args[1].eq_ignore_ascii_case("--h")
        || args[1].eq_ignore_ascii_case("--help")
    {
        eprintln!(
            "Usage: {} <CARGO_LOCK_FILE_PATH>",
            // SAFETY: arg 0 will be valid binary path on any supported shell/OS
            Path::new(&args[0]).file_name().unwrap().to_str().unwrap()
        );
        process::exit(1);
    }

    let cargo_lock_path = Path::new(&args[1]);

    if !cargo_lock_path.exists() || !cargo_lock_path.is_file() {
        eprintln!("Error: invalid file path");
        process::exit(1);
    }

    match cargo_lock_path.file_name() {
        Some(file_name) => {
            if file_name != OsStr::new("Cargo.toml") {
                eprintln!("Warning: input file is not Cargo.toml");
            }
        }
        None => {
            eprintln!("Warning: cannot determine filename");
        }
    }

    println!("Checking: {}", cargo_lock_path.display());

    if let Err(e) = Policy::new(cargo_lock_path)
        .expect("Invalid manifest path")
        .allowed_category_publishers([("cryptography", "rustcrypto")].into_iter())
        .no_duplicate_crate_categories(["cryptography"].into_iter())
        .run()
    {
        eprintln!("{e:?}");
        process::exit(1);
    }

    process::exit(0);
}
