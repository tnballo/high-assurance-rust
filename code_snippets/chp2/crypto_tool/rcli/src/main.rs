// ANCHOR: full_imports
use clap::Parser;
use rc4::Rc4;
use std::fs::File;
use std::io::prelude::{Read, Seek, Write};
// ANCHOR_END: full_imports

// ANCHOR: clap_args
/// RC4 file en/decryption
#[derive(Parser, Debug)]
struct Args {
    /// Name of file to en/decrypt
    #[clap(short, long, required = true, value_name = "FILE_NAME")]
    file: String,

    /// En/Decryption key (hexadecimal bytes)
    #[clap(
        short,
        long,
        required = true,
        value_name = "HEX_BYTES",
        min_values = 5,
        max_values = 256
    )]
    key: Vec<String>,
}
// ANCHOR_END: clap_args

// ANCHOR: cli_main
fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let mut contents = Vec::new();

    // Convert key strings to byte array
    let key_bytes = args
        .key
        .iter()
        .map(|s| s.trim_start_matches("0x"))
        .map(|s| u8::from_str_radix(s, 16).expect("Invalid key hex byte!"))
        .collect::<Vec<u8>>();

    // Validation note:
    // `Args` enforces (5 <= key_bytes.len() && key_bytes.len() <= 256)

    // Open the file for both reading and writing
    let mut file = File::options().read(true).write(true).open(&args.file)?;

    // Read all file contents into memory
    file.read_to_end(&mut contents)?;

    // En/decrypt file contents in-memory
    Rc4::apply_keystream_static(&key_bytes, &mut contents);

    // Overwrite existing file with the result
    file.rewind()?; // "Seek" to start of file stream
    file.write_all(&contents)?;

    // Print success message
    println!("Processed {}", args.file);

    // Return success
    Ok(())
}
// ANCHOR_END: cli_main
