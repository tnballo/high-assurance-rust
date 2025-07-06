// ANCHOR: builder_demo
use supplychain_policy::Policy;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR var not set");
    let manifest_path = std::path::PathBuf::from(manifest_dir).join("Cargo.toml");

    Policy::new(&manifest_path)
        .expect("Invalid manifest path")
        .allowed_category_publishers([("cryptography", "rustcrypto")].into_iter())
        .no_duplicate_crate_categories(["cryptography"].into_iter())
        .run()
        .unwrap()
}
// ANCHOR_END: builder_demo
