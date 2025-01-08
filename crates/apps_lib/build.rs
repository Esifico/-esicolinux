use std::fs::File;
use std::io::Write;
use std::str;

use cargo_metadata::MetadataCommand;
use git2::{DescribeFormatOptions, DescribeOptions, Repository};

/// Path to the .proto source files, relative to `apps_lib` directory
const PROTO_SRC: &str = "./proto";

fn main() {
    // Discover the repository version, if it exists
    println!("cargo:rerun-if-changed=../../.git");
    let describe_opts = DescribeOptions::new();
    let mut describe_format = DescribeFormatOptions::new();
    describe_format.dirty_suffix("-dirty");
    let repo = Repository::discover(".").ok();
    let describe = match &repo {
        Some(repo) => repo.describe(&describe_opts).ok(),
        None => None,
    };
    let version_string = match describe {
        Some(describe) => describe.format(Some(&describe_format)).ok(),
        None => None,
    };
    let mut version_rs =
        File::create("./version.rs").expect("cannot write version");
    let pre = "pub fn namada_version() -> &'static str { \"";
    let post = "\" }";
    match version_string {
        Some(version_string) => {
            version_rs
                .write_all(pre.as_bytes())
                .expect("cannot write version");
            version_rs
                .write_all(version_string.as_bytes())
                .expect("cannot write version");
            version_rs
                .write_all(post.as_bytes())
                .expect("cannot write version");
        }
        None => {
            // Get the version from the apps crate
            let version = MetadataCommand::new()
                .no_deps()
                .other_options(vec![
                    "--locked".to_string(),
                    "--offline".to_string(),
                ])
                .manifest_path("../apps/Cargo.toml")
                .exec()
                .ok()
                .and_then(|metadata| {
                    metadata
                        .packages
                        .into_iter()
                        .find(|package| package.name == "namada_apps")
                })
                .map(|package| format!("v{}", package.version))
                .unwrap();
            version_rs
                .write_all(pre.as_bytes())
                .expect("cannot write version");
            version_rs
                .write_all(version.as_bytes())
                .expect("cannot write version");
            version_rs
                .write_all(post.as_bytes())
                .expect("cannot write version");
        }
    };

    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed={}", PROTO_SRC);
}
