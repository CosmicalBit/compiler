use crate::parser::program::ast_generate;
use clap::Parser;
use std::{fs, path::PathBuf};
mod lexer;
mod parser;

#[derive(Debug, Parser)]
#[command(about = "Compile a source file")]
struct Args {
    /// Source file to compile
    file: PathBuf,

    /// Optimization level (0-3)
    #[arg(short = 'O', long = "opt-level", value_parser = clap::value_parser!(u8).range(0..=3), default_value_t = 0)]
    opt_level: u8,
}

fn main() {
    let args = Args::parse();
    let contents = fs::read_to_string(&args.file)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", args.file.display()));

    let lexemes = lexer::scanner::create_lexemes(&contents);
    let tokens = lexer::classify::classify_scanner(&lexemes);

    let ast = ast_generate(&tokens).unwrap();
}
