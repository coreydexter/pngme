use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "pngme")]
pub struct ApplicationArguments {
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    /// Add a message to a specified PNG file
    #[structopt(name = "encode")]
    Encode(Encode),
    /// Read a message from a specified PNG file
    #[structopt(name = "decode")]
    Decode(Decode),
    /// Remove a message from a specified PNG file
    #[structopt(name = "remove")]
    Remove(Remove),
}

#[derive(StructOpt, Debug)]
pub struct Encode {
    /// The input PNG file
    #[structopt(parse(from_os_str))]
    pub file_path: PathBuf,
    /// The 4 letter chunk type to use, eg teSt
    pub chunk_type: String,
    /// The message to encode
    pub message: String,
    /// Where to write the updated PNG to. If not provided, will overwrite the input PNG
    #[structopt(parse(from_os_str))]
    pub output_file: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
pub struct Decode {
    /// The input PNG file
    #[structopt(parse(from_os_str))]
    pub file_path: PathBuf,
    /// The 4 letter chunk type to search for, eg teSt
    pub chunk_type: String,
}

#[derive(StructOpt, Debug)]
pub struct Remove {
    /// The input PNG file
    #[structopt(parse(from_os_str))]
    pub file_path: PathBuf,
    /// The 4 letter chunk type to remove, eg teSt. Will only remove the first chunk of this type found
    pub chunk_type: String,
    /// Where to write the updated PNG to. If not provided, will overwrite the input PNG
    #[structopt(parse(from_os_str))]
    pub output_file: Option<PathBuf>,
}
