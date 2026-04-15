use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use dbc_codegen2::{app::App, codegen::config::CodegenConfig};
use dbc_codegen2::utils::Language;

struct GeneratedFileGuard {
    path: PathBuf,
    original: Option<Vec<u8>>,
}

impl GeneratedFileGuard {
    fn new(path: PathBuf) -> std::io::Result<Self> {
        let original = if path.exists() {
            Some(std::fs::read(&path)?)
        } else {
            None
        };

        Ok(Self { path, original })
    }
}

impl Drop for GeneratedFileGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(content) => {
                let _ = std::fs::write(&self.path, content);
            }
            None => {
                let _ = std::fs::remove_file(&self.path);
            }
        }
    }
}

fn _dbc_files() -> Vec<PathBuf> {
    let base = Path::new("./shared-test-files");
    let mut files = Vec::new();

    fn visit(dir: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                visit(&path, files);
            } else if path.extension().map(|e| e == "dbc").unwrap_or(false) {
                files.push(path);
            }
        }
    }

    visit(base, &mut files);
    files
}

fn _run_codegen(input: &Path) -> Result<()> {
    let output_path = "../data/generated.rs";
    let input_str = input
        .to_str()
        .context("Invalid UTF-8 in input path")?
        .to_string();

    let config = CodegenConfig {
        input: input_str,
        output: output_path.into(),
        lang: Language::Rust,
        no_enum_other: false,
        no_enum_dedup: false,
        zero_zero_range_allows_all: false,
    };

    App::run(config).context("codegen failed")?;
    Ok(())
}

fn _cargo_check_data_crate() -> Result<()> {
    let status = Command::new("cargo")
        .args(["check", "-p", "codegen-validator"])
        .status()
        .context("failed to run cargo check")?;

    if !status.success() {
        anyhow::bail!("cargo check failed");
    }

    Ok(())
}

#[test]
fn test_all_dbc_files() -> Result<()> {
    let _guard = GeneratedFileGuard::new("../data/generated.rs".into())
        .context("Failed to create file guard")?;

    let files = _dbc_files();
    println!("Running {} tests", files.len());

    let mut failures = Vec::new();

    for file in files {
        println!("Testing {:?}", file);

        let result = std::panic::catch_unwind(|| {
            (|| -> Result<()> {
                _run_codegen(&file)
                    .with_context(|| format!("Codegen failed for {:?}", file))?;

                Command::new("cargo")
                    .args(["clean", "-p", "codegen-validator"])
                    .status()
                    .ok();

                _cargo_check_data_crate()
                    .with_context(|| format!("Compilation failed for {:?}", file))?;

                Ok(())
            })()
        });

        match result {
            Ok(Ok(())) => (),
            Ok(Err(e)) => {
                failures.push((file, format!("{:#}", e)));
            }
            Err(panic) => {
                let panic_msg = if let Some(s) = panic.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic".to_string()
                };

                failures.push((file, format!("PANIC: {}", panic_msg)));
            }
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== FAILURES ===\n");

        for (file, err) in &failures {
            eprintln!("File: {:?}", file);
            eprintln!("{}\n", err);
        }

        anyhow::bail!("{} test(s) failed", failures.len());
    }

    Ok(())
}
