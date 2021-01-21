mod cli_args;

use crate::cli_args::Opt;
use anyhow::{anyhow, Error};
use distance_bytes::GameObject;
use std::{
    fs,
    fs::File,
    io,
    io::{Read, Write},
};

fn main() -> Result<(), Error> {
    let args = cli_args::get();
    let output_format = get_output_format(&args)?;

    let input = match args.bytes_file {
        Some(path) => fs::read(&path)?,
        None => {
            let mut buf = Vec::new();
            io::stdin().read_to_end(&mut buf)?;

            buf
        }
    };

    let game_object = distance_bytes::deserialize_game_object(&input)?;

    let output_fn: fn(Box<dyn Write>, &GameObject) -> Result<(), Error> = match output_format {
        OutputFormat::Json => |writer, value| Ok(serde_json::to_writer(writer, value)?),
        OutputFormat::Yaml => |writer, value| Ok(serde_yaml::to_writer(writer, value)?),
    };

    let writer: Box<dyn Write> = match args.output {
        Some(path) => Box::new(File::open(path)?),
        None => Box::new(io::stdout()),
    };

    output_fn(writer, &game_object)?;

    Ok(())
}

#[derive(Debug)]
enum OutputFormat {
    Json,
    Yaml,
}

fn get_output_format(args: &Opt) -> Result<OutputFormat, Error> {
    match (args.json, args.yaml) {
        (false, false) | (false, true) => Ok(OutputFormat::Yaml),
        (true, false) => Ok(OutputFormat::Json),
        (true, true) => Err(anyhow!("multiple output format flags were given")),
    }
}
