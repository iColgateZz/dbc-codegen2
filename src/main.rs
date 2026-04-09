use can_dbc::Dbc as ParsedDbc;
use clap::{Parser, Subcommand};
use dbc_codegen2::{DbcFile, app::App, codegen::config::CodegenConfig, ir::IRBuilder, utils::Language};
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
};

#[derive(Parser)]
#[command(name = "dbc-codegen")]
#[command(about = "DBC code generator", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Parse a DBC file and print parsed output
    Parse {
        /// Input file path
        input: String,
        /// Output file
        #[arg(short, long, default_value = "data/parsed_can_dbc.txt")]
        output: String,
    },

    /// Show intermediate representation
    Ir {
        /// Input file path
        input: String,
        /// Output file
        #[arg(short, long, default_value = "data/ir.txt")]
        output: String,
    },

    /// Generate code from DBC
    Gen {
        /// Input file path
        input: String,
        /// Output file
        #[arg(short, long, default_value = "data/generated.rs")]
        output: String,
        /// Target language for code generation
        #[arg(short, long, value_enum, default_value = "rust")]
        lang: Language,
        /// Disable _Other variant for signal value enums
        #[arg(long, default_value = "false")]
        no_enum_other: bool,
        /// Disable signal value enum (SVE) deduplication.
        /// By default, SVEs with same names & value descriptions
        /// are treated as one enum.
        #[arg(long, default_value = "false")]
        no_enum_dedup: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Parse { input, output } => {
            let dbc = parse_dbc_file(&input);
            if let Err(e) = write_parsed_dbc(dbc, &output) {
                eprintln!("Error parsing dbc: {e}");
            }
        }

        Command::Ir { input, output } => {
            let dbc = parse_dbc_file(&input);
            let ir = IRBuilder::to_ir(dbc);
            if let Err(e) = write_ir(ir, &output) {
                eprintln!("Error writing IR: {e}");
            }
        }

        Command::Gen { 
            input,
            output, 
            lang, 
            no_enum_other,
            no_enum_dedup,
        } => {
            let config = CodegenConfig {
                input: input,
                output,
                lang,
                no_enum_other,
                no_enum_dedup,
            };

            let _ = App::run(config);
        }
    }
}

fn write_parsed_dbc(dbc: ParsedDbc, output: &str) -> std::io::Result<()> {
    let output_file = File::create(output)?;
    let mut writer = BufWriter::new(output_file);

    writeln!(writer, "// Generated file with parsed can_dbc structs")?;
    writeln!(writer, "{:#?}", dbc.messages)?;

    Ok(())
}

fn write_ir(ir: DbcFile, output: &str) -> std::io::Result<()> {
    let output_file = File::create(output)?;
    let mut writer = BufWriter::new(output_file);

    writeln!(writer, "// Generated file with IR")?;
    writeln!(writer, "{:#?}", ir)?;

    Ok(())
}

pub fn parse_dbc_file(file_path: &str) -> ParsedDbc {
    let data = fs::read_to_string(file_path).expect("Unable to read input file");
    ParsedDbc::try_from(data.as_str()).unwrap()
}
