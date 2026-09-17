//! RFC-033 Task 13 — package API docs schema v2: `schema_version` 2 iff any
//! emitted item uses M2 syntax; M2 items carry `body_source` (the fmt-canonical
//! body text) and omit the `insts`/`calls`/`nets` summary. v1 documents stay
//! byte-identical (tests/apidocs.rs pins those).

use cohdl::emit::docsjson::{render, PackageMeta, Rendered};
use cohdl::pipeline::{check_files_in_with_deps, Checked};

fn std_files() -> Vec<(String, String)> {
    let std_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/std/src");
    let mut entries: Vec<_> = std::fs::read_dir(&std_dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "cohdl"))
        .collect();
    entries.sort();
    entries
        .into_iter()
        .map(|p| {
            (
                format!("std/{}", p.file_name().unwrap().to_string_lossy()),
                std::fs::read_to_string(&p).unwrap(),
            )
        })
        .collect()
}

fn std_version() -> String {
    let std_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/std");
    let (_, manifest) = cohdl::project::peek_manifest(&std_dir).unwrap();
    manifest.version.expect("std manifest pins a version")
}

fn check_pkg(package: &str, files: &[(&str, &str)]) -> Checked {
    let mut all = std_files();
    all.extend(files.iter().map(|(n, c)| (n.to_string(), c.to_string())));
    let checked =
        check_files_in_with_deps(package, &["std".to_string()], &all, None).expect("pipeline runs");
    assert!(
        !checked.diags.has_errors(),
        "fixture must check cleanly:\n{}",
        checked.diags.render(&checked.sm)
    );
    checked
}

fn docs_for(package: &str, version: &str, files: &[(&str, &str)]) -> Rendered {
    let checked = check_pkg(package, files);
    render(
        &checked,
        &PackageMeta {
            name: package,
            version,
            description: None,
            license: None,
            repository: None,
        },
        &[DepMeta {
            name: "std".to_string(),
            version: std_version(),
            src_layout: true,
        }],
    )
}

use cohdl::emit::docsjson::DepMeta;

#[test]
fn m2_package_gets_schema_v2_and_body_source() {
    let docs = docs_for(
        "m2pkg",
        "0.1.0",
        &[(
            "src/main.cohdl",
            "pub fn bank<const N: Int = 2>(p: Pin) { for x: i in 0..N { net _: p } }\n",
        )],
    );
    assert!(
        docs.json.contains("\"schema_version\": 2"),
        "schema 2:\n{}",
        docs.json
    );
    // The document is pretty-printed multi-line: assert the const-Int bound
    // pieces rather than a single-line JSON substring.
    assert!(
        docs.json.contains("\"bound\": {") && docs.json.contains("\"const\": \"Int\""),
        "const Int bound:\n{}",
        docs.json
    );
    assert!(
        docs.json.contains("\"default\": \"2\"") || docs.json.contains("\"default\": 2"),
        "numeric default:\n{}",
        docs.json
    );
    assert!(
        docs.json.contains("body_source"),
        "M2 item carries body_source:\n{}",
        docs.json
    );
    // The body_source is the fmt-canonical body text.
    assert!(
        docs.json.contains("for x: i in 0..N {"),
        "body_source contains the canonical loop text:\n{}",
        docs.json
    );
    // An M2 item omits the insts/calls/nets summary keys.
    let item_start = docs.json.find("\"name\": \"bank\"").expect("bank item");
    let item_end = docs.json[item_start..]
        .find("\"body_source\"")
        .map(|i| item_start + i)
        .unwrap_or(docs.json.len());
    let item = &docs.json[item_start..item_end];
    assert!(!item.contains("\"nets\""), "M2 item omits nets:\n{item}");
}

#[test]
fn legacy_package_stays_schema_v1() {
    let docs = docs_for(
        "plainpkg",
        "0.1.0",
        &[("src/main.cohdl", "pub fn plain(p: Pin) { net _: p }\n")],
    );
    assert!(
        docs.json.contains("\"schema_version\": 1"),
        "schema 1:\n{}",
        docs.json
    );
    assert!(
        !docs.json.contains("body_source"),
        "no M2 items → no body_source:\n{}",
        docs.json
    );
}
