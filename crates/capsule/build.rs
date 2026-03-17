// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::path::PathBuf;

fn read_trimmed_if_exists(path: &PathBuf) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(s) => Some(s.trim().to_string()),
        Err(_) => None,
    }
}

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));

    // When building from the git repo, we embed VERSION/BUILD.
    // When building from crates.io (or any other packaging), those files are not present;
    // fall back to the crate version from Cargo metadata.
    let maybe_repo_root = manifest_dir.parent().and_then(|p| p.parent()).map(|p| p.to_path_buf());
    let (repo_version, repo_build) = if let Some(repo_root) = maybe_repo_root {
        let version_path = repo_root.join("VERSION");
        let build_path = repo_root.join("BUILD");

        // If these files exist in the local checkout, track them.
        if version_path.exists() {
            println!("cargo:rerun-if-changed={}", version_path.display());
        }
        if build_path.exists() {
            println!("cargo:rerun-if-changed={}", build_path.display());
        }

        (read_trimmed_if_exists(&version_path), read_trimmed_if_exists(&build_path))
    } else {
        (None, None)
    };

    let pkg_version = std::env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION");

    let version = repo_version.unwrap_or_else(|| pkg_version.clone());
    let build = repo_build;

    if version.is_empty() {
        panic!("version is empty");
    }

    // Minimal sanity: version should be semver-ish; allow digits, dots, and hyphens.
    if !version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
    {
        panic!("version contains invalid characters; got '{version}'");
    }

    let semver = match build.as_deref() {
        Some(build) => {
            if build.is_empty() {
                panic!("BUILD is empty");
            }
            if !build.chars().all(|c| c.is_ascii_digit()) {
                panic!("BUILD must contain only digits; got '{build}'");
            }
            format!("{version}+build.{build}")
        }
        None => version.clone(),
    };

    println!("cargo:rustc-env=CAPSULE_VERSION={version}");
    println!("cargo:rustc-env=CAPSULE_SEMVER={semver}");
    if let Some(build) = build.as_deref() {
        println!("cargo:rustc-env=CAPSULE_BUILD={build}");
    } else {
        println!("cargo:rustc-env=CAPSULE_BUILD=");
    }
}
