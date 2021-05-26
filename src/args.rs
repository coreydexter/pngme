use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct ApplicationArguments {
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    #[structopt(name = "encode")]
    Encode(Encode),
    #[structopt(name = "decode")]
    Decode(Decode),
    #[structopt(name = "remove")]
    Remove(Remove),
}

#[derive(StructOpt, Debug)]
pub struct Encode {
    #[structopt(parse(from_os_str))]
    pub file_path: PathBuf,
    pub chunk_type: String,
    pub message: String,
    #[structopt(parse(from_os_str))]
    pub output_file: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
pub struct Decode {
    #[structopt(parse(from_os_str))]
    pub file_path: PathBuf,
    pub chunk_type: String,
}

#[derive(StructOpt, Debug)]
pub struct Remove {
    #[structopt(parse(from_os_str))]
    pub file_path: PathBuf,
    pub chunk_type: String,
    #[structopt(parse(from_os_str))]
    pub output_file: Option<PathBuf>,
}
