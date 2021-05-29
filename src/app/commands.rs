use crate::args::IdentifyText;
use crate::args::{Decode, Encode, Remove};
use anyhow::Context;
use lib_pngme::chunk::Chunk;
use lib_pngme::png::Png;

pub fn execute_encode(args: Encode) -> anyhow::Result<()> {
    let mut png = Png::from_file(&args.file_path)
        .with_context(|| format!("Failed to load PNG file {:?}", args.file_path))?;

    let file_path = args.file_path;
    png.append_chunk(Chunk::new(args.chunk_type, args.message.into_bytes()));

    if let Some(output_file) = args.output_file {
        println!("Writing out file to {:?}", output_file);
        png.write_file(&output_file)
            .with_context(|| format!("Failed to write file {:?}", output_file))
    } else {
        println!("Writing out file to {:?}", file_path);
        png.write_file(&file_path)
            .with_context(|| format!("Failed to write file {:?}", file_path))
    }
}

pub fn execute_decode(args: Decode) -> anyhow::Result<()> {
    let png = Png::from_file(&args.file_path)
        .with_context(|| format!("Failed to load PNG file {:?}", args.file_path))?;

    let chunk = png.chunk_by_type(&args.chunk_type);

    match chunk {
        Some(chunk) => {
            let data = chunk.data_as_string().with_context(|| {
                format!(
                    "Failed to decode message from {} as string",
                    args.chunk_type
                )
            })?;
            println!("{}", data);
        }
        None => {
            eprintln!("Failed to find a chunk of type {}", args.chunk_type)
        }
    };

    Ok(())
}

pub fn execute_remove(args: Remove) -> anyhow::Result<()> {
    let mut png = Png::from_file(&args.file_path)
        .with_context(|| format!("Failed to load PNG file {:?}", args.file_path))?;

    png.remove_chunk(&args.chunk_type)?;

    if let Some(output_file) = args.output_file {
        println!("Writing out file to {:?}", output_file);
        png.write_file(&output_file)
            .with_context(|| format!("Failed to write file {:?}", output_file))
    } else {
        println!("Writing out file to {:?}", args.file_path);
        png.write_file(&args.file_path)
            .with_context(|| format!("Failed to write file {:?}", args.file_path))
    }
}

pub fn execute_identify_text(args: IdentifyText) -> anyhow::Result<()> {
    let png = Png::from_file(&args.file_path)
        .with_context(|| format!("Failed to load PNG file {:?}", args.file_path))?;

    for (index, chunk) in png.chunks().iter().enumerate() {
        match chunk.data_as_string() {
            Ok(data) => {
                if data.len() > 0 {
                    println!("{} - {} - {}", index, chunk.chunk_type(), data);
                }
            }
            Err(_) => {}
        }
    }

    Ok(())
}
