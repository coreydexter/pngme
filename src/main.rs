mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

use crate::args::ApplicationArguments;
use commands::{execute_decode, execute_encode, execute_remove};
use structopt::StructOpt;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() {
    let args = ApplicationArguments::from_args();

    match args.command {
        args::Command::Encode(args) => execute_encode(args),
        args::Command::Decode(args) => execute_decode(args),
        args::Command::Remove(args) => execute_remove(args),
    };
}
