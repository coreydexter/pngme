use crate::args::{Decode, Encode, Remove};
use lib_pngme::chunk::Chunk;
use lib_pngme::png::Png;

pub fn execute_encode(args: Encode) {
    let mut png = Png::from_file(&args.file_path).expect("Failed to load PNG file");

    png.append_chunk(Chunk::new(args.chunk_type, args.message.into_bytes()));

    if let Some(output_file) = args.output_file {
        println!("Writing out file to {:?}", output_file);
        png.write_file(&output_file)
            .expect(&format!("Failed to write out file to {:?}", output_file));
    } else {
        println!("Writing out file to {:?}", args.file_path);
        png.write_file(&args.file_path)
            .expect(&format!("Failed to write out file to {:?}", args.file_path));
    }
}

pub fn execute_decode(args: Decode) {
    let png = Png::from_file(&args.file_path).expect("Failed to load PNG file");

    if let Some(chunk) = png.chunk_by_type(&args.chunk_type) {
        println!(
            "{}",
            chunk
                .data_as_string()
                .expect("Failed to decode message from string")
        );
    }
}

pub fn execute_remove(args: Remove) {
    let mut png = Png::from_file(&args.file_path).expect("Failed to load PNG file");

    png.remove_chunk(&args.chunk_type);

    if let Some(output_file) = args.output_file {
        png.write_file(&output_file)
            .expect(&format!("Failed to write out file to {:?}", output_file));
    } else {
        png.write_file(&args.file_path)
            .expect(&format!("Failed to write out file to {:?}", args.file_path));
    }
}
