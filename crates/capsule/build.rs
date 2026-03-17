// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::path::PathBuf;

fn read_trimmed(path: &PathBuf) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
        .trim()
        .to_string()
}

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| panic!("unexpected crate layout; cannot locate repo root from {}", manifest_dir.display()))
        .to_path_buf();

    let version_path = repo_root.join("VERSION");
    let build_path = repo_root.join("BUILD");

    println!("cargo:rerun-if-changed={}", version_path.display());
    println!("cargo:rerun-if-changed={}", build_path.display());

    let version = read_trimmed(&version_path);
    let build = read_trimmed(&build_path);

    if version.is_empty() {
        panic!("VERSION is empty");
    }
    if build.is_empty() {
        panic!("BUILD is empty");
    }

    // Minimal sanity: VERSION should be digits+dots, BUILD digits.
    if !version.chars().all(|c| c.is_ascii_digit() || c == '.') {
        panic!("VERSION must contain only digits and dots; got '{version}'");
    }
    if !build.chars().all(|c| c.is_ascii_digit()) {
        panic!("BUILD must contain only digits; got '{build}'");
    }

    let semver = format!("{version}+build.{build}");

    println!("cargo:rustc-env=CAPSULE_VERSION={version}");
    println!("cargo:rustc-env=CAPSULE_BUILD={build}");
    println!("cargo:rustc-env=CAPSULE_SEMVER={semver}");
}
