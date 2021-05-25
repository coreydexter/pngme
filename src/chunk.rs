use std::convert::TryFrom;
use std::fmt::Display;
use std::io::Read;
use std::str::FromStr;

use crate::chunk_type::ChunkType;
use crate::{Error, Result};

#[derive(Debug)]
enum ChunkError {
    InvalidCRCValue,
    RemainingBytes,
    LengthTooLarge,
}

impl std::error::Error for ChunkError {}
impl std::fmt::Display for ChunkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChunkError::InvalidCRCValue => {
                write!(f, "Provided CRC value does not match calculated CRC value")
            }
            ChunkError::LengthTooLarge => write!(f, "Length exceeds allow range"),
            ChunkError::RemainingBytes => {
                write!(f, "There were bytes remaining, length is likely incorrect")
            }
        }
    }
}

pub struct Chunk {
    // By the PNG 1.2 specification length must be less than
    // 2^31.
    length: u32,
    chunk_type: ChunkType,
    chunk_data: Vec<u8>,
    // A 4-byte CRC (Cyclic Redundancy Check)
    crc: u32,
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, chunk_data: Vec<u8>) -> Chunk {
        // TODO how can we provide a single slice with chunk_type and chunk_data
        // and no new vec creation?
        let mut crc_data = Vec::with_capacity(4 + chunk_data.len());
        crc_data.extend(chunk_type.bytes().iter());
        crc_data.extend(chunk_data.iter());

        let crc = calculate_crc(&crc_data[..]);
        Chunk {
            length: chunk_data.len() as u32,
            chunk_type: chunk_type,
            chunk_data: chunk_data,
            crc,
        }
    }

    pub fn from_strings(chunk_type: &str, chunk_data: &str) -> Result<Chunk> {
        let chunk_type = ChunkType::from_str(chunk_type)?;

        Ok(Chunk::new(chunk_type, chunk_data.bytes().collect()))
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        &self.chunk_data[..]
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }

    pub fn data_as_string(&self) -> Result<String> {
        let str = String::from_utf8(self.chunk_data.clone())
            .expect("ahh whatever failed to parse utf-8 bytes as a string");

        Ok(str)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        unimplemented!("as_bytes")
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.data_as_string().expect("ahh todo convert error type")
        )
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        let orig_value = value;
        let mut value = value;

        let mut length = [0 as u8; 4];
        value.read_exact(&mut length)?;

        let length = u32::from_be_bytes(length);

        if length > (1 << 31) {
            return Err(Box::new(ChunkError::LengthTooLarge));
        }

        let mut chunk_type_buf = [0 as u8; 4];
        value.read_exact(&mut chunk_type_buf)?;
        let chunk_type = ChunkType::try_from(chunk_type_buf)?;

        let mut chunk_data: Vec<u8> = vec![0; length as usize];
        value.read_exact(&mut chunk_data)?;

        let mut crc = [0 as u8; 4];
        value.read_exact(&mut crc)?;
        let crc = u32::from_be_bytes(crc);

        if !value.is_empty() {
            return Err(Box::new(ChunkError::RemainingBytes));
        }

        // The CRC is calculated from the bytes of the chunk_type and chunk_data
        // So skip the first 4 bytes (i.e length), and the last 4 bytes (i.e provided CRC)
        let calculated_crc = calculate_crc(&orig_value[4..orig_value.len() - 4]);
        if calculated_crc != crc {
            return Err(Box::new(ChunkError::InvalidCRCValue));
        }

        Ok(Chunk {
            length,
            chunk_type,
            chunk_data,
            crc,
        })
    }
}

fn calculate_crc(value: &[u8]) -> u32 {
    // Based off the implementation of
    // http://www.libpng.org/pub/png/spec/1.2/PNG-CRCAppendix.html
    let mut crc: u32 = 0xffffffff; // All 1's

    let crc_table = create_crc_table();

    for v in value.iter() {
        crc = crc_table[(crc as u8 ^ v) as usize] ^ (crc >> 8);
    }

    crc ^ 0xffffffff
}

fn create_crc_table() -> [u32; 256] {
    let mut crc_table = [0; 256];

    for index in 0..crc_table.len() {
        let mut c = index as u32;
        for _ in 0..8 {
            if c & 1 == 1 {
                c = 0xedb88320 ^ (c >> 1);
            } else {
                c = c >> 1;
            }
        }
        crc_table[index] = c;
    }

    crc_table
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        println!("{:?}", chunk_data);

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type.to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        // 1 less than it should be, thereby causing an error
        let crc: u32 = 2882656334 - 1;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
