#![deny(missing_docs)]

//! A demo supply-chain policy builder.

// ANCHOR: builder_impl_1
use cargo_metadata::{CargoOpt, Metadata, MetadataCommand, Package, semver::Version};
use std::{
    cell::OnceCell,
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
};

/// A [`Policy`] violation.
/// Note: error variants do expose/re-export error enums from 3rd-party crates.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum PolicyViolationError {
    DuplicateCrateVersions(Vec<String>),
    DisallowedCategoryPublisher(String, String),
    MetadataReadError(String),
}

/// A builder for supply-chain policies.
#[derive(Default)]
pub struct Policy {
    // Path to `Cargo.toml` we're analyzing
    manifest_path: PathBuf,
    // Workaround for `OnceCell::get_or_try_init` being nightly-only in Rust 1.88
    cargo_metadata_result: OnceCell<Result<Metadata, PolicyViolationError>>,
    // {category}
    // `String`s lower-cased at construction time
    no_dup_cats: Option<BTreeSet<String>>,
    // category: {publisher}
    // `String`s lower-cased at construction time
    cat_pubs: Option<BTreeMap<String, BTreeSet<String>>>,
}

impl Policy {
    /// Create a new policy, construct with path to workspace or crate-specific `Cargo.toml`.
    pub fn new<P>(manifest_path: P) -> Result<Policy, std::io::Error>
    where
        P: AsRef<Path>,
    {
        let manifest_path = fs::canonicalize(manifest_path)?;
        Ok(Self {
            manifest_path,
            ..Default::default()
        })
    }

    /// Rule 1 (Category-specific Trusted Publishers):
    /// Ensure that a given category only contains crates from a fixed set of trusted publishers.
    /// Assumes input iterator format `(category_1, publisher_1)...(category_n, publisher_n)`.
    /// More then one publisher per category is supported.
    pub fn allowed_category_publishers<I, S>(mut self, cat_pubs: I) -> Policy
    where
        I: Iterator<Item = (S, S)>,
        S: Into<String>,
    {
        let mut cat_pubs = cat_pubs.peekable();
        if cat_pubs.peek().is_some() {
            let mut cat_map = BTreeMap::new();
            for (c, p) in cat_pubs {
                cat_map
                    .entry(c.into().to_ascii_lowercase())
                    .or_insert(BTreeSet::new())
                    .insert(p.into().to_ascii_lowercase());
            }
            self.cat_pubs = Some(cat_map);
        } else {
            self.cat_pubs = None;
        }

        self
    }
    // ANCHOR_END: builder_impl_1

    /// Rule 2 (Category-specific No Duplicates):
    /// Specify categories which cannot have duplicate crates.
    pub fn no_duplicate_crate_categories<I, S>(mut self, cats: I) -> Policy
    where
        I: Iterator<Item = S>,
        S: Into<String>,
    {
        let mut cats = cats.peekable();
        if cats.peek().is_some() {
            self.no_dup_cats = Some(cats.map(|s| s.into().to_ascii_lowercase()).collect());
        } else {
            self.no_dup_cats = None;
        }

        self
    }

    // ANCHOR: builder_impl_2
    /// Evaluate a built policy against a given workspace/crate.
    pub fn run(&self) -> Result<(), PolicyViolationError> {
        self.run_allowed_category_publishers()?;
        self.run_no_duplicate_crate_categories()?;
        Ok(())
    }
    // ANCHOR_END: builder_impl_2

    // ANCHOR: policy_impl
    /// Collect dependency metadata for the entire workspace with all features enabled.
    fn metadata(&self) -> Result<&Metadata, PolicyViolationError> {
        let meta_result = self.cargo_metadata_result.get_or_init(|| {
            MetadataCommand::new()
                .manifest_path(&self.manifest_path)
                .features(CargoOpt::AllFeatures)
                .exec()
                .map_err(|e| PolicyViolationError::MetadataReadError(e.to_string()))
        });

        meta_result.as_ref().map_err(|e| e.to_owned())
    }

    /// Get repo's publisher by parsing its URL.
    // SECURITY: `dep.authors` isn't reliable - anyone can set any value in their crate's `Cargo.toml`.
    fn get_repo_publisher(dep: &Package) -> Result<String, PolicyViolationError> {
        let Some(repo_url) = dep
            .repository
            .as_ref()
            .and_then(|url| url::Url::parse(url).ok())
        else {
            return Err(PolicyViolationError::MetadataReadError(format!(
                "Missing or invalid repo URL for crate '{}'",
                dep.name
            )));
        };

        // If `repo_url` == "https://github.com/RustCrypto/AEADs/tree/master/aes-gcm"
        // Then `repo_publisher` == "RustCrypto"
        let Some(repo_publisher) = repo_url.path_segments().and_then(|mut path| path.next()) else {
            return Err(PolicyViolationError::MetadataReadError(format!(
                "Missing publisher name for repo URL '{repo_url}'"
            )));
        };

        Ok(repo_publisher.to_string())
    }

    /// Run category-specific trusted publishers check.
    fn run_allowed_category_publishers(&self) -> Result<(), PolicyViolationError> {
        let Some(ref cat_pubs) = self.cat_pubs else {
            return Ok(());
        };

        let metadata = self.metadata()?;

        // ID direct dependencies
        let direct_deps = metadata
            .packages
            .iter()
            .filter(|pkg| pkg.manifest_path.as_path() == self.manifest_path)
            .map(|pkg| &pkg.dependencies)
            .flatten()
            .collect::<Vec<_>>();

        // Get full crate info for each ID-ed direct dependency
        let direct_dep_crates = metadata
            .packages
            .iter()
            .filter(|pkg| direct_deps.iter().any(|dep| dep.name == *pkg.name));

        // Find disallowed category-specific publishers, if any
        for dep_crate in direct_dep_crates {
            for cat in &dep_crate.categories {
                if let Some(expected_pubs) = cat_pubs.get(&cat.to_ascii_lowercase()) {
                    let actual_publisher = Self::get_repo_publisher(dep_crate)?.to_lowercase();
                    if !expected_pubs.contains(&actual_publisher) {
                        return Err(PolicyViolationError::DisallowedCategoryPublisher(
                            cat.clone(),
                            actual_publisher,
                        ));
                    }
                }
            }
        }

        Ok(())
    }
    // ANCHOR_END: policy_impl

    /// Run category-specific no duplicates check.
    fn run_no_duplicate_crate_categories(&self) -> Result<(), PolicyViolationError> {
        type DepToVerMap<'a> = HashMap<&'a String, BTreeSet<&'a Version>>;

        let Some(ref no_dup_cats) = self.no_dup_cats else {
            return Ok(());
        };

        let metadata = self.metadata()?;

        // dependency : {dependency_versions}
        let dep_to_dup_versions: DepToVerMap = metadata
            .packages
            .iter()
            // 3rd-party dep
            .filter(|dep| !dep.manifest_path.starts_with(&metadata.workspace_root))
            // Category we've specified no duplicates for
            .filter(|dep| {
                dep.categories
                    .iter()
                    .any(|x| no_dup_cats.iter().any(|y| y.eq_ignore_ascii_case(x)))
            })
            .fold(HashMap::new(), |mut map: DepToVerMap, dep| {
                map.entry(&dep.name).or_default().insert(&dep.version);
                map
            })
            .into_iter()
            // Duplicate versions in-tree
            .filter(|(_, versions)| versions.len() >= 2)
            .collect();

        if !dep_to_dup_versions.is_empty() {
            let dup_crates = dep_to_dup_versions
                .iter()
                .flat_map(|(dep_name, versions)| {
                    let mut list = Vec::with_capacity(versions.len());
                    for v in versions {
                        list.push(format!("{dep_name} {v}"));
                    }
                    list
                })
                .collect();

            return Err(PolicyViolationError::DuplicateCrateVersions(dup_crates));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Policy;

    #[test]
    fn test_repo_owners_extraction() {
        use cargo_metadata::{PackageBuilder, PackageId, camino::Utf8PathBuf, semver::Version};
        use cargo_util_schemas::manifest::PackageName;

        let mock_dep = PackageBuilder::new(
            PackageName::new("mock-dep".to_string()).unwrap(),
            Version::parse("0.1.0").unwrap(),
            PackageId {
                repr: "mock-dep 0.1.0 (path+file:///path/to/mock-dep)".to_string(),
            },
            Utf8PathBuf::from("path/to/mock-dep"),
        )
        .authors(vec!["author1".to_string(), "author2".to_string()])
        .repository(Some("https://github.com/RustCrypto/AEADs".to_string()))
        .build()
        .unwrap();

        assert_eq!(
            "RustCrypto".to_string(),
            Policy::get_repo_publisher(&mock_dep).unwrap(),
        );
    }

    #[test]
    fn test_cargo_metadata_api() {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR var not set");
        let manifest_path = std::path::PathBuf::from(manifest_dir).join("Cargo.toml");

        let test_policy = Policy::new(&manifest_path).expect("Invalid manifest path");
        let test_metadata = test_policy.metadata().expect("Failed to get metadata");

        let third_party_crates: Vec<_> = test_metadata
            .packages
            .iter()
            // 3rd-party dep
            .filter(|dep| !dep.manifest_path.starts_with(&test_metadata.workspace_root))
            .map(|p| p.name.to_string())
            .collect();

        assert!(third_party_crates.contains(&"cargo_metadata".to_string()));
        assert!(!third_party_crates.contains(&"supplychain_policy".to_string()));

        let direct_deps: Vec<_> = test_metadata
            .packages
            .iter()
            .filter(|pkg| pkg.manifest_path.as_path() == test_policy.manifest_path)
            .map(|pkg| &pkg.dependencies)
            .flatten()
            .map(|dep| dep.name.to_string())
            .collect();

        assert!(direct_deps.contains(&"cargo_metadata".to_string()));
        assert!(!direct_deps.contains(&"supplychain_policy".to_string()));
        assert!(!direct_deps.contains(&"camino".to_string()));
    }
}
