use std::{
    convert::TryFrom,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use crate::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChunkTypeError {
    #[error("Character length `{0}` of chunk type must be exactly 4")]
    InvalidCharacterLength(usize),
    #[error("Invalid character at `{0}` in input, must be an ASCII upper-case or lower-case value, not `{1}`")]
    InvalidCharacter(usize, u8),
}

// A structure representing the Chunk Type header of a PNG chunk.
// A Chunk Type is a 4 character string where each character is an ASCII letter,
// with upper-case and lower-case having different meanings.
// Eg the Chunk Type `RuSt` is different to `Rust`
// Refer to section 3.2 on the PNG Specification 1.2 for reference of details.
// TODO write a brief description on creating a ChunkType
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ChunkType {
    bytes: [u8; 4],
}

impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        self.bytes.clone()
    }
    pub fn is_valid(&self) -> Result<(), Box<ChunkTypeError>> {
        // For convenience in description and in examining PNG files,
        // type codes are restricted to consist of uppercase and lowercase
        // ASCII letters (A-Z and a-z, or 65-90 and 97-122 decimal)
        let bad_byte = self
            .bytes
            .iter()
            .enumerate()
            .filter(|(_, &v)| !ChunkType::is_ascii(v))
            .next();

        match bad_byte {
            Some((i, &v)) => Err(Box::new(ChunkTypeError::InvalidCharacter(i, v))),
            None => Ok(()),
        }
    }

    fn is_ascii(v: u8) -> bool {
        (v >= 65 && v <= 90) || (v >= 97 && v <= 122)
    }

    pub fn is_critical(&self) -> bool {
        // Ancillary bit: bit 5 of first byte
        // 0 (uppercase) = critical, 1 (lowercase) = ancillary.
        is_bit_zero(self.bytes[0], 5)
    }
    pub fn is_public(&self) -> bool {
        // Private bit: bit 5 of second byte
        // 0 (uppercase) = public, 1 (lowercase) = private.
        is_bit_zero(self.bytes[1], 5)
    }
    pub fn is_reserved_bit_valid(&self) -> bool {
        // Reserved bit: bit 5 of third byte
        // Must be 0 (uppercase) in files conforming to this version of PNG.
        !is_bit_zero(self.bytes[2], 5)
    }
    pub fn is_safe_to_copy(&self) -> bool {
        // Safe-to-copy bit: bit 5 of fourth byte
        // 0 (uppercase) = unsafe to copy, 1 (lowercase) = safe to copy.
        !is_bit_zero(self.bytes[3], 5)
    }
}

fn is_bit_zero(input: u8, bit: u8) -> bool {
    assert!(bit <= 8);

    input & (1 << bit) == 0
}

impl FromStr for ChunkType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.as_bytes();
        match s.len() {
            4 => TryFrom::try_from([s[0], s[1], s[2], s[3]]),
            len => Err(Box::new(ChunkTypeError::InvalidCharacterLength(len))),
        }
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = Error;

    fn try_from(bytes: [u8; 4]) -> Result<Self, Self::Error> {
        let chunk = ChunkType { bytes };

        match chunk.is_valid() {
            Ok(_) => Ok(chunk),
            Err(e) => Err(e),
        }
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            self.bytes[0] as char,
            self.bytes[1] as char,
            self.bytes[2] as char,
            self.bytes[3] as char
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid().is_ok());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(chunk.is_valid().is_ok());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
