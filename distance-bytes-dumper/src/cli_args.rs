use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(parse(from_os_str))]
    pub bytes_file: Option<PathBuf>,

    #[structopt(long)]
    pub json: bool,

    #[structopt(long)]
    pub yaml: bool,

    #[structopt(short, long, parse(from_os_str))]
    pub output: Option<PathBuf>,
}

pub fn get() -> Opt {
    Opt::from_args()
}
