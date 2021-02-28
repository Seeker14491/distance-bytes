mod cli_args;

use crate::cli_args::Opt;
use anyhow::{ensure, Error};
use distance_bytes::GameObject;
use std::fs::File;
use std::io;
use std::io::{Cursor, Read, Write};

fn main() -> Result<(), Error> {
    color_backtrace::install();
    tracing_subscriber::fmt::init();

    let args = cli_args::get();
    let output_format = get_output_format(&args)?;

    let game_object = match args.bytes_file {
        Some(path) => GameObject::read_from_file(&path)?,
        None => {
            let mut buf = Vec::new();
            io::stdin().read_to_end(&mut buf)?;
            GameObject::read_from_reader(Cursor::new(buf))?
        }
    };

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
    const DEFAULT_FORMAT: OutputFormat = OutputFormat::Yaml;

    let flags = [args.json, args.yaml];
    let format_flag_count = flags.iter().filter(|x| **x).count();
    ensure!(
        format_flag_count <= 1,
        "multiple output format flags were given"
    );

    let format = if args.json {
        OutputFormat::Json
    } else if args.yaml {
        OutputFormat::Yaml
    } else {
        DEFAULT_FORMAT
    };

    Ok(format)
}
