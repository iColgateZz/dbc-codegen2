use can_dbc::Dbc as ParsedDbc;
use std::fs;
use std::path::PathBuf;

use crate::codegen;
use crate::codegen::config::CodegenConfig;
use crate::middle_end::nodes::{AttachMessageSignalUsage, AttachSignalValueEnumType, CheckEnumVariants, CheckMessageSignalUsage, CheckSignalLayoutValidity, CheckSignalScalingArithmeticSafety, CheckSignalPhysicalRangeRepresentable, CheckUniqueMessageIds, CheckUnsupportedMultiplexing, CheckZeroZeroRanges, ComputeBitvecPositions, DeduplicateSignalValueEnums, Diagnostics, InferSignalTypes, PrefixSignalValueEnumName, SanitizeMessageNames, SanitizeSVENames, SanitizeSignalNames};
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
    pub fn run(config: CodegenConfig) -> anyhow::Result<()> {
        let mut parsed_dbcs = config.inputs.iter().map(|input| {
            let data = fs::read_to_string(input)
                .unwrap_or_else(|e| panic!("Unable to read input file `{input}`: {e}"));

            ParsedDbc::try_from(data.as_str())
                .unwrap_or_else(|e| panic!("Unable to parse input file `{input}`: {e}"))
        });

        let first = parsed_dbcs
            .next()
            .expect("At least one input file is required");

        let merged_parsed_dbc = parsed_dbcs.fold(first, |mut acc, dbc| {
            merge_parsed_dbcs(&mut acc, dbc);
            acc
        });
        let mut dbc = IRBuilder::to_ir(merged_parsed_dbc);

        TransformationPipeline::new()
            .add(ComputeBitvecPositions)
            .add(AttachMessageSignalUsage)
            .add(InferSignalTypes)
            .run(&mut dbc);

        let mut diagnostics = Diagnostics::default();
        CheckPipeline::new()
            .add(CheckZeroZeroRanges {zero_zero_range_allows_all: config.zero_zero_range_allows_all})
            .add(CheckUniqueMessageIds)
            .add(CheckSignalLayoutValidity)
            .add(CheckMessageSignalUsage)
            .add(CheckUnsupportedMultiplexing)
            .add(CheckEnumVariants)
            .add(CheckSignalPhysicalRangeRepresentable {zero_zero_range_allows_all: config.zero_zero_range_allows_all})
            .add(CheckSignalScalingArithmeticSafety)
            .run(&dbc, &mut diagnostics);

        diagnostics.emit();

        if diagnostics.has_errors() {
            anyhow::bail!("En error was found during validation phase!");
        }

        TransformationPipeline::new()
            .add(SanitizeSignalEnumVariantNames)
            .add(DeduplicateSignalValueEnums {dedup_enabled: !config.no_enum_dedup})
            .add(PrefixSignalValueEnumName {dedup_enabled: !config.no_enum_dedup})
            .add(AttachSignalValueEnumType)
            .add(SanitizeMessageNames)
            .add(SanitizeSVENames)
            .add(SanitizeSignalNames)
            .run(&mut dbc);

        let code = match config.lang {
            Language::Rust => codegen::rust::RustGen::generate(&dbc, &config),
            Language::Cpp => codegen::cpp::CppGen::generate(&dbc, &config),
        };

        let ext = config.lang.file_extension();
        let out = PathBuf::from(config.output).with_extension(ext);
        std::fs::write(out, code)?;

        Ok(())
    }
}

// most of the symbols are not used
fn merge_parsed_dbcs(dst: &mut ParsedDbc, mut src: ParsedDbc) {
    // if dst.version == Default::default() {
    //     dst.version = src.version;
    // }

    // if dst.bit_timing.is_none() {
    //     dst.bit_timing = src.bit_timing.take();
    // }

    // dst.new_symbols.append(&mut src.new_symbols);
    dst.nodes.append(&mut src.nodes);
    // dst.value_tables.append(&mut src.value_tables);
    dst.messages.append(&mut src.messages);
    // dst.message_transmitters.append(&mut src.message_transmitters);
    // dst.environment_variables.append(&mut src.environment_variables);
    // dst.environment_variable_data.append(&mut src.environment_variable_data);
    // dst.signal_types.append(&mut src.signal_types);
    dst.comments.append(&mut src.comments);
    // dst.attribute_definitions.append(&mut src.attribute_definitions);
    // dst.relation_attribute_definitions.append(&mut src.relation_attribute_definitions);
    // dst.attribute_defaults.append(&mut src.attribute_defaults);
    // dst.relation_attribute_defaults.append(&mut src.relation_attribute_defaults);
    // dst.relation_attribute_values.append(&mut src.relation_attribute_values);
    // dst.attribute_values_database.append(&mut src.attribute_values_database);
    // dst.attribute_values_node.append(&mut src.attribute_values_node);
    // dst.attribute_values_message.append(&mut src.attribute_values_message);
    // dst.attribute_values_signal.append(&mut src.attribute_values_signal);
    // dst.attribute_values_env.append(&mut src.attribute_values_env);
    dst.value_descriptions.append(&mut src.value_descriptions);
    // dst.signal_type_refs.append(&mut src.signal_type_refs);
    // dst.signal_groups.append(&mut src.signal_groups);
    dst.signal_extended_value_type_list.append(&mut src.signal_extended_value_type_list);
    dst.extended_multiplex.append(&mut src.extended_multiplex);
}
