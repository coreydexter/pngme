mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

use crate::args::ApplicationArguments;
use structopt::StructOpt;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() {
    let args = ApplicationArguments::from_args();
}
