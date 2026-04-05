use can_dbc::Dbc as ParsedDbc;
use std::fs;

use crate::codegen;
use crate::middle_end::nodes::{ComputeBitvecPositions, InferSignalTypes};
use crate::utils::Language;
use crate::{
    ir::IRBuilder,
    middle_end::{
        nodes::SanitizeSignalEnumVariantNames, pipeline::transform_pipeline::TransformationPipeline,
    },
};

pub struct App;

impl App {
    pub fn convert(input_path: &str, language: Language) -> String {
        let data = fs::read_to_string(input_path).expect("Unable to read input file");
        let mut dbc = IRBuilder::to_ir(ParsedDbc::try_from(data.as_str()).unwrap());

        //TODO: give user options to add new nodes/remove nodes
        TransformationPipeline::new()
            .add(ComputeBitvecPositions)
            .add(SanitizeSignalEnumVariantNames)
            .add(InferSignalTypes)
            .run(&mut dbc);

        match language {
            Language::Rust => codegen::rust::RustGen::generate(&dbc),
            Language::Cpp => codegen::cpp::CppGen::generate(&dbc),
        }
    }
}
