#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use anyhow::{Context, Result};
    use dbc_codegen2::utils::Language;
    use dbc_codegen2::{app::CodegenPipeline, codegen::config::CodegenConfig};

    const DBC_DIR: &str = "./test-files";
    const GENERATED_RUST_FILE: &str = "../data/generated.rs";
    const GENERATED_CPP_FILE: &str = "../data/generated.hpp";

    struct GeneratedFileGuard {
        path: PathBuf,
        original: Option<Vec<u8>>,
    }

    impl GeneratedFileGuard {
        fn new(path: impl Into<PathBuf>) -> Result<Self> {
            let path = path.into();
            let original = path.exists().then(|| fs::read(&path)).transpose()?;

            Ok(Self { path, original })
        }
    }

    impl Drop for GeneratedFileGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(content) => {
                    let _ = fs::write(&self.path, content);
                }
                None => {
                    let _ = fs::remove_file(&self.path);
                }
            }
        }
    }

    fn dbc_files() -> Vec<PathBuf> {
        fn visit(dir: &Path, files: &mut Vec<PathBuf>) {
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();

                if path.is_dir() {
                    visit(&path, files);
                } else if path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("dbc"))
                {
                    files.push(path);
                }
            }
        }

        let mut files = Vec::new();
        visit(Path::new(DBC_DIR), &mut files);
        files.sort();
        files
    }

    fn should_pass(file: &Path) -> bool {
        file.starts_with(Path::new(DBC_DIR).join("currently-work"))
    }

    fn generate(input: &Path, output: &str, lang: Language, separate: bool) -> Result<()> {
        let mut config = CodegenConfig {
            inputs: vec![
                input
                    .to_str()
                    .context("invalid UTF-8 in input path")?
                    .into(),
            ],
            output: output.into(),
            lang,
            no_enum_other: true,
            no_enum_dedup: false,
            zero_zero_range_allows_all: false,
            rust_code_injections: HashMap::new(),
            cpp_code_injections: HashMap::new(),
            generate_tests: true,
            separate,
        };

        config.add_rust_code_injection(
            dbc_codegen2::RustCodeInjectionPoint::Getter,
            "#[inline(always)]",
        );

        config.add_rust_code_injection(
            dbc_codegen2::RustCodeInjectionPoint::Setter,
            "#[inline(always)]",
        );

        CodegenPipeline::run(&config).context("codegen failed")
    }

    fn run_quiet(command: &mut Command, label: &str) -> Result<()> {
        let output = command
            .output()
            .with_context(|| format!("failed to run {label}"))?;

        if !output.status.success() {
            anyhow::bail!(
                "{} failed with status {}\nstdout:\n{}\nstderr:\n{}",
                label,
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    fn rust_fixture(file: &Path) -> Result<()> {
        generate(file, GENERATED_RUST_FILE, Language::Rust, false)
            .with_context(|| format!("Codegen failed for {:?}", file))?;

        let mut check = Command::new("cargo");
        check.args(["check", "-p", "data"]);
        run_quiet(&mut check, "cargo check")
            .with_context(|| format!("Compilation failed for {:?}", file))?;

        let mut test = Command::new("cargo");
        test.args(["test", "-p", "data"]);
        run_quiet(&mut test, "cargo test").with_context(|| format!("Test failed for {:?}", file))
    }

    fn write_cpp_runner(dir: &Path) -> Result<PathBuf> {
        let runner = dir.join("generated_tests.cpp");
        fs::write(
            &runner,
            "#include \"generated.hpp\"\n\nint main() {\n  generated_tests::run_all();\n  return 0;\n}\n",
        )?;
        Ok(runner)
    }

    fn cpp_fixture(file: &Path, runner: &Path, binary: &Path) -> Result<()> {
        generate(file, GENERATED_CPP_FILE, Language::Cpp, false)
            .with_context(|| format!("Codegen failed for {:?}", file))?;

        let compiler = env::var("CXX").unwrap_or_else(|_| "c++".to_string());
        let include_dir = Path::new(GENERATED_CPP_FILE).parent().unwrap();
        let mut compile = Command::new(&compiler);
        compile
            .arg("-std=c++23")
            .arg("-I")
            .arg(include_dir)
            .arg(runner)
            .arg("-o")
            .arg(binary);
        run_quiet(&mut compile, &compiler)
            .with_context(|| format!("Compilation failed for {:?}", file))?;

        let mut run = Command::new(binary);
        run_quiet(&mut run, "generated C++ tests")
            .with_context(|| format!("Test failed for {:?}", file))
    }

    fn cpp_separate_fixture(
        file: &Path,
        output: &Path,
        runner: &Path,
        binary: &Path,
    ) -> Result<()> {
        generate(
            file,
            output.to_str().context("invalid UTF-8 in output path")?,
            Language::Cpp,
            true,
        )
        .with_context(|| format!("Codegen failed for {:?}", file))?;

        let output_dir = output.parent().unwrap();
        let output_stem = output
            .file_name()
            .and_then(|stem| stem.to_str())
            .context("invalid UTF-8 in output file name")?;
        let expected_message_headers = [
            ("driver_heartbeat_msg", "DriverHeartbeatMsg"),
            ("io_debug_msg", "IoDebugMsg"),
            ("motor_cmd_msg", "MotorCmdMsg"),
            ("motor_status_msg", "MotorStatusMsg"),
            ("sensor_sonars_msg", "SensorSonarsMsg"),
        ];
        let expected_header_count = expected_message_headers.len() + 2;
        let mut split_headers = fs::read_dir(output_dir)?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("hpp"))
            .collect::<Vec<_>>();
        split_headers.sort();
        anyhow::ensure!(
            split_headers.len() == expected_header_count,
            "expected {} generated C++ headers, got {}: {:?}",
            expected_header_count,
            split_headers.len(),
            split_headers
        );

        let aggregate_header = output.with_extension("hpp");
        let common_header = output.with_file_name(format!("{}_common.hpp", output_stem));
        anyhow::ensure!(
            aggregate_header.exists(),
            "missing aggregate header {:?}",
            aggregate_header
        );
        anyhow::ensure!(
            common_header.exists(),
            "missing common header {:?}",
            common_header
        );

        let aggregate_contents = fs::read_to_string(&aggregate_header)?;
        anyhow::ensure!(
            aggregate_contents.contains(&format!("#include \"{}_common.hpp\"", output_stem)),
            "aggregate header should include the common header"
        );

        for (message_file_suffix, class_name) in expected_message_headers {
            let header_name = format!("{}_{}.hpp", output_stem, message_file_suffix);
            let header = output.with_file_name(&header_name);
            anyhow::ensure!(header.exists(), "missing message header {:?}", header);

            let contents = fs::read_to_string(&header)?;
            anyhow::ensure!(
                contents.contains(&format!("class {}", class_name)),
                "{:?} should contain class {}",
                header,
                class_name
            );
            anyhow::ensure!(
                aggregate_contents.contains(&format!("#include \"{}\"", header_name)),
                "aggregate header should include {}",
                header_name
            );
        }

        let compiler = env::var("CXX").unwrap_or_else(|_| "c++".to_string());
        let mut compile = Command::new(&compiler);
        compile
            .arg("-std=c++23")
            .arg("-I")
            .arg(output_dir)
            .arg(runner)
            .arg("-o")
            .arg(binary);
        run_quiet(&mut compile, &compiler)
            .with_context(|| format!("Compilation failed for {:?}", file))?;

        let mut run = Command::new(binary);
        run_quiet(&mut run, "generated C++ tests")
            .with_context(|| format!("Test failed for {:?}", file))
    }

    fn run_fixtures(
        generated_file: &str,
        mut run_fixture: impl FnMut(&Path) -> Result<()>,
    ) -> Result<()> {
        let _guard = GeneratedFileGuard::new(generated_file)?;
        let mut failures = Vec::new();

        for file in dbc_files() {
            match (
                run_fixture(&file).map_err(|e| format!("{e:#}")),
                should_pass(&file),
            ) {
                (Ok(()), true) | (Err(_), false) => {}
                (Ok(()), false) => failures.push((
                    file,
                    "Expected generation pipeline to fail, but it passed".to_string(),
                )),
                (Err(err), true) => failures.push((file, err)),
            }
        }

        if failures.is_empty() {
            return Ok(());
        }

        eprintln!("\n=== FAILURES ===\n");
        for (file, err) in &failures {
            eprintln!("File: {file:?}");
            eprintln!("{err}\n");
        }

        anyhow::bail!("{} test(s) failed", failures.len());
    }

    #[test]
    fn test_all_dbc_files_rust() -> Result<()> {
        run_fixtures(GENERATED_RUST_FILE, rust_fixture)
    }

    #[test]
    fn test_all_dbc_files_cpp() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let runner = write_cpp_runner(temp_dir.path())?;
        let binary = temp_dir.path().join("generated_tests");

        run_fixtures(GENERATED_CPP_FILE, |file| {
            cpp_fixture(file, &runner, &binary)
        })
    }

    #[test]
    fn test_cpp_separate_generation() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let runner = write_cpp_runner(temp_dir.path())?;
        let binary = temp_dir.path().join("generated_tests");
        let output = temp_dir.path().join("generated");
        let input = Path::new(DBC_DIR).join("currently-work/example.dbc");

        cpp_separate_fixture(&input, &output, &runner, &binary)
    }
}
