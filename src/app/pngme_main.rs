mod args;
mod commands;

use crate::args::ApplicationArguments;
use commands::{
    execute_decode, execute_encode, execute_identify_text, execute_print, execute_remove,
};
use structopt::StructOpt;

fn main() -> anyhow::Result<()> {
    let args = ApplicationArguments::from_args();

    match args.command {
        args::Command::Encode(args) => execute_encode(args),
        args::Command::Decode(args) => execute_decode(args),
        args::Command::Remove(args) => execute_remove(args),
        args::Command::IdentifyText(args) => execute_identify_text(args),
        args::Command::Print(args) => execute_print(args),
    }
}
