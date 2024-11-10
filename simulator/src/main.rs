mod error;
mod voltmeter;

use clap::{
    builder::{OsStringValueParser, TypedValueParser},
    Parser,
};
use error::ConductorSimResult;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Input {
    Stdin,
    Path(PathBuf),
}

fn input_parser() -> impl TypedValueParser<Value = Input> {
    OsStringValueParser::new().map(|value| {
        if value == "-" {
            Input::Stdin
        } else {
            Input::Path(PathBuf::from(value))
        }
    })
}

fn delimiter_parser() -> impl TypedValueParser<Value = u8> {
    OsStringValueParser::new().try_map(|value| {
        let length = value.len();

        if length < 1 || length > 1 {
            Err(clap::Error::new(clap::error::ErrorKind::InvalidValue))
        } else {
            Ok(value.as_encoded_bytes()[0])
        }
    })
}

#[derive(Debug, Clone, Parser)]
pub struct Command {
    #[clap(value_parser = input_parser())]
    #[arg(short, long)]
    pub file: Input,

    #[clap(value_parser = delimiter_parser())]
    #[arg(short, long, default_value = ",")]
    pub delimiter: u8,

    #[arg(short, long)]
    pub target: String,

    #[arg(short, long)]
    pub sample_rate: u16,
}

fn main() -> ConductorSimResult<()> {
    let command = Command::parse();

    voltmeter::voltmeter(command)
}
