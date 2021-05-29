use std::convert::TryFrom;
use std::fmt::Display;
use std::io;
use std::io::Read;
use std::str::FromStr;
use std::string::FromUtf8Error;
use thiserror::Error;

use crate::chunk_type::ChunkType;
use crate::chunk_type::ChunkTypeError;

pub type ChunkResult = Result<Chunk, ChunkError>;

#[derive(Error, Debug)]
pub enum ChunkError {
    #[error("Provided CRC value `{0}` does match calculated CRC value `{1}`")]
    InvalidCRCValue(u32, u32),
    #[error("There were bytes `{0}` remaining, length is likely incorrect")]
    RemainingBytes(usize),
    #[error("Length of `{0}` exceeds number of bytes `{1}")]
    LengthTooLarge(usize, usize),
    #[error("There weren't enough bytes `{0}` to satify the specified chunks length `{1}`")]
    NotEnoughBytes(usize, u32),
    #[error("Data is not a valid UTF-8 string")]
    DataNotValidUtf8 {
        #[from]
        source: FromUtf8Error,
    },
    #[error("Failed to read")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("Invalid ChunkType string")]
    InvalidChunk {
        #[from]
        source: ChunkTypeError,
    },
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

    pub fn from_strings(chunk_type: &str, chunk_data: &str) -> ChunkResult {
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

    pub fn data_as_string(&self) -> Result<String, ChunkError> {
        let str = String::from_utf8(self.chunk_data.clone())?;

        Ok(str)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.length
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type.to_string().as_bytes().iter())
            .chain(self.chunk_data.iter())
            .chain(self.crc.to_be_bytes().iter())
            .copied()
            .collect()
    }

    pub fn next_chunk(stream: &[u8]) -> Result<&[u8], ChunkError> {
        if stream.len() < 4 {
            // Minimum length for a chunk is 12 - 4 for length, 4 for type, 0 for data, 4 for CRC
            return Err(ChunkError::NotEnoughBytes(stream.len(), 12));
        }

        let orig_stream = stream;
        let mut stream = stream;

        let mut length = [0 as u8; 4];
        stream.read_exact(&mut length)?;
        let length = u32::from_be_bytes(length);

        // Now we know the data length, we can determine the length of this chunk
        // 4 bytes for length, 4 bytes for type, length bytes for data, 4 bytes for CRC
        let chunk_length = (4 + 4 + length + 4) as usize;

        if chunk_length > orig_stream.len() {
            return Err(ChunkError::LengthTooLarge(chunk_length, orig_stream.len()));
        }

        Ok(&orig_stream[..chunk_length])
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data_as_string().map_err(|_| std::fmt::Error)?)
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkError;

    fn try_from(value: &[u8]) -> ChunkResult {
        let orig_value = value;
        let mut value = value;

        let mut length = [0 as u8; 4];
        value.read_exact(&mut length)?;

        let length = u32::from_be_bytes(length);

        if length > (1 << 31) {
            return Err(ChunkError::LengthTooLarge(length as usize, 1 << 31));
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
            return Err(ChunkError::RemainingBytes(value.len()));
        }

        // The CRC is calculated from the bytes of the chunk_type and chunk_data
        // So skip the first 4 bytes (i.e length), and the last 4 bytes (i.e provided CRC)
        let calculated_crc = calculate_crc(&orig_value[4..orig_value.len() - 4]);
        if calculated_crc != crc {
            return Err(ChunkError::InvalidCRCValue(crc, calculated_crc));
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
    fn test_chunk_as_bytes() {
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

        let chunk_bytes = chunk.as_bytes();
        assert_eq!(chunk_bytes, chunk_data);
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
