use supplychain_policy::Policy;

#[test]
fn test_self_analysis() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR var not set");
    let manifest_path = std::path::PathBuf::from(manifest_dir).join("Cargo.toml");

    // Applies to `url` crate dependency
    Policy::new(&manifest_path)
        .expect("Invalid manifest path")
        .allowed_category_publishers([("parser-implementations", "servo")].into_iter())
        .allowed_category_publishers([("web-programming", "servo")].into_iter())
        .no_duplicate_crate_categories(["parser-implementations"].into_iter())
        .no_duplicate_crate_categories(["web-programming"].into_iter())
        .run()
        .unwrap()
}
