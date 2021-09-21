mod cli_args;

use crate::cli_args::OutputFormat;
use anyhow::Error;
use distance_bytes::GameObject;
use std::io;
use std::io::{Cursor, Read, Write};

fn main() -> Result<(), Error> {
    color_backtrace::install();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args = cli_args::get();

    let mut game_object = {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf)?;
        GameObject::read_from_reader(Cursor::new(buf))?
    };

    match args.format {
        OutputFormat::Json => {
            serde_json::to_writer(io::stdout(), &game_object)?;
        }
        OutputFormat::Yaml => {
            serde_yaml::to_writer(io::stdout(), &game_object)?;
        }
        OutputFormat::Bytes => {
            let mut buf = Cursor::new(Vec::new());
            game_object.write_to_writer(&mut buf)?;
            io::stdout().write_all(&buf.into_inner())?;
        }
    };

    Ok(())
}
