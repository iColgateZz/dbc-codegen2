use can_dbc::Dbc as ParsedDbc;
use std::fs;

use crate::codegen;
use crate::middle_end::nodes::SanitizeMessageNames;
use crate::{
    DbcFile,
    middle_end::{
        nodes::{AttachSignalValueEnums, SanitizeSignalEnumVariantNames},
        pipeline::transform_pipeline::TransformationPipeline,
    },
};

//TODO: this definetely has to have some flags
//      At least to chose between rust and c++
pub struct App;

impl App {
    pub fn convert(input_path: &str) -> String {
        let data = fs::read_to_string(input_path).expect("Unable to read input file");
        let mut dbc = DbcFile::from_dbc(ParsedDbc::try_from(data.as_str()).unwrap());

        //TODO: give user options to add new nodes/remove nodes
        TransformationPipeline::new()
            .add(SanitizeMessageNames)
            .add(SanitizeSignalEnumVariantNames)
            .add(AttachSignalValueEnums)
            .run(&mut dbc);

        codegen::rust::RustGen::generate(&dbc.messages)
    }
}
