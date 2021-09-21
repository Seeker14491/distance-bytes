use clap::arg_enum;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug)]
    pub enum OutputFormat {
        Json,
        Yaml,
        Bytes,
    }
}

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(long, possible_values = &OutputFormat::variants(), case_insensitive = true)]
    pub format: OutputFormat,
}

pub fn get() -> Opt {
    Opt::from_args()
}
