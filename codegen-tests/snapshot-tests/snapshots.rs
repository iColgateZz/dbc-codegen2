use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use dbc_codegen2::app::CodegenPipeline;
use dbc_codegen2::codegen::config::CodegenConfig;
use dbc_codegen2::utils::Language;

const ALL_CASES_DBC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/snapshot-tests/all_cases.dbc");

fn generated_path(output_base: &Path, lang: Language) -> PathBuf {
    output_base.with_extension(lang.file_extension())
}

fn generate_snapshot_fixture(lang: Language) -> String {
    let tempdir = tempfile::tempdir().expect("failed to create temp output directory");
    let output_base = tempdir.path().join("generated");
    let generated_path = generated_path(&output_base, lang.clone());
    let output = output_base
        .to_str()
        .expect("temp output path should be valid UTF-8")
        .to_string();

    let config = CodegenConfig {
        inputs: vec![ALL_CASES_DBC.to_string()],
        output,
        lang,
        enum_other: true,
        enum_dedup: true,
        allow_unrestricted_ranges: false,
        rust_code_injections: HashMap::new(),
        cpp_code_injections: HashMap::new(),
        generate_tests: false,
        separate: false,
    };

    CodegenPipeline::run(config).expect("codegen should succeed for snapshot fixture");

    fs::read_to_string(generated_path).expect("generated file should be readable")
}

#[test]
fn snapshots_rust_generation() {
    insta::assert_snapshot!("rust_all_cases", generate_snapshot_fixture(Language::Rust));
}

#[test]
fn snapshots_cpp_generation() {
    insta::assert_snapshot!("cpp_all_cases", generate_snapshot_fixture(Language::Cpp));
}
