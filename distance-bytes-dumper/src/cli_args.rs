use clap::Parser;

#[derive(Parser, Debug)]
pub struct Opt {
    #[clap(long, value_enum, ignore_case = true)]
    pub format: OutputFormat,
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum OutputFormat {
    Json,
    Yaml,
    Bytes,
}

pub fn get() -> Opt {
    Opt::parse()
}
