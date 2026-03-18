use can_dbc::Dbc as ParsedDbc;
use std::fs;

use crate::codegen;
use crate::middle_end::nodes::InferSignalType;
use crate::{
    ir::IRBuilder,
    middle_end::{
        nodes::SanitizeSignalEnumVariantNames,
        pipeline::transform_pipeline::TransformationPipeline,
    },
};

//TODO: this definetely has to have some flags
//      At least to chose between rust and c++
pub struct App;

impl App {
    pub fn convert(input_path: &str) -> String {
        let data = fs::read_to_string(input_path).expect("Unable to read input file");
        let mut dbc = IRBuilder::to_ir(ParsedDbc::try_from(data.as_str()).unwrap());

        //TODO: give user options to add new nodes/remove nodes
        TransformationPipeline::new()
            .add(SanitizeSignalEnumVariantNames)
            .add(InferSignalType)
            .run(&mut dbc);

        codegen::rust::RustGen::generate(&dbc)
    }
}
