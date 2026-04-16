use can_dbc::Dbc as ParsedDbc;
use std::fs;
use std::path::PathBuf;
use std::process::exit;

use crate::codegen;
use crate::codegen::config::CodegenConfig;
use crate::middle_end::nodes::{AttachSignalValueEnumType, CheckZeroZeroRanges, ComputeBitvecPositions, DeduplicateSignalValueEnums, Diagnostics, InferSignalTypes, PrefixSignalValueEnumName, SanitizeMessageNames};
use crate::middle_end::pipeline::check_pipeline::CheckPipeline;
use crate::utils::Language;
use crate::{
    ir::IRBuilder,
    middle_end::{
        nodes::SanitizeSignalEnumVariantNames, pipeline::transform_pipeline::TransformationPipeline,
    },
};

pub struct App;

impl App {
    pub fn run(config: CodegenConfig) -> std::io::Result<()> {
        let data = fs::read_to_string(&config.input).expect("Unable to read input file");
        let mut dbc = IRBuilder::to_ir(ParsedDbc::try_from(data.as_str()).unwrap());

        //TODO: give user options to add new nodes/remove nodes
        TransformationPipeline::new()
            .add(ComputeBitvecPositions)
            .add(SanitizeSignalEnumVariantNames)
            .add(InferSignalTypes)
            .add(DeduplicateSignalValueEnums {dedup_enabled: !config.no_enum_dedup})
            .add(PrefixSignalValueEnumName {dedup_enabled: !config.no_enum_dedup})
            .add(AttachSignalValueEnumType)
            .add(SanitizeMessageNames)
            .run(&mut dbc);

        let mut diagnostics = Diagnostics::default();
        CheckPipeline::new()
            .add(CheckZeroZeroRanges {zero_zero_range_allows_all: config.zero_zero_range_allows_all})
            .run(&dbc, &mut diagnostics);

        diagnostics.emit();

        if diagnostics.has_errors() {
            exit(1);
        }

        let code = match config.lang {
            Language::Rust => codegen::rust::RustGen::generate(&dbc, &config),
            Language::Cpp => codegen::cpp::CppGen::generate(&dbc),
        };

        let ext = config.lang.file_extension();
        let out = PathBuf::from(config.output).with_extension(ext);
        std::fs::write(out, code)?;

        Ok(())
    }
}
