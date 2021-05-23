use std::{
    convert::TryFrom,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use crate::Error;

#[derive(Debug)]
enum ChunkTypeError {
    InvalidCharacterLength,
    InvalidCharacter,
}

impl std::error::Error for ChunkTypeError {}
impl fmt::Display for ChunkTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ChunkTypeError::InvalidCharacterLength => {
                write!(f, "Character length of chunk type must be exactly 4")
            }
            ChunkTypeError::InvalidCharacter => {
                write!(
                    f,
                    "Invalid character in input, must be 65 <= v <= 90 or 97 <= v <= 122"
                )
            }
        }
    }
}

// A structure representing the Chunk Type header of a PNG chunk.
// A Chunk Type is a 4 character string where each character is an ASCII letter,
// with upper-case and lower-case having different meanings.
// Eg the Chunk Type `RuSt` is different to `Rust`
// Refer to section 3.2 on the PNG Specification 1.2 for reference of details.
#[derive(Debug, PartialEq, Eq)]
struct ChunkType {
    bytes: [u8; 4],
}

impl ChunkType {
    fn bytes(&self) -> [u8; 4] {
        self.bytes.clone()
    }
    fn is_valid(&self) -> bool {
        //  For convenience in description and in examining PNG files,
        // type codes are restricted to consist of uppercase and lowercase
        // ASCII letters (A-Z and a-z, or 65-90 and 97-122 decimal)
        self.bytes
            .iter()
            .all(|&val| (val >= 65 && val <= 90) || (val >= 97 && val <= 122))
    }
    fn is_critical(&self) -> bool {
        // Ancillary bit: bit 5 of first byte
        // 0 (uppercase) = critical, 1 (lowercase) = ancillary.
        is_bit_zero(self.bytes[0], 5)
    }
    fn is_public(&self) -> bool {
        // Private bit: bit 5 of second byte
        // 0 (uppercase) = public, 1 (lowercase) = private.
        is_bit_zero(self.bytes[1], 5)
    }
    fn is_reserved_bit_valid(&self) -> bool {
        // Reserved bit: bit 5 of third byte
        // Must be 0 (uppercase) in files conforming to this version of PNG.
        !is_bit_zero(self.bytes[2], 5)
    }
    fn is_safe_to_copy(&self) -> bool {
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
            _ => Err(Box::new(ChunkTypeError::InvalidCharacterLength)),
        }
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = Error;

    fn try_from(bytes: [u8; 4]) -> Result<Self, Self::Error> {
        let chunk = ChunkType { bytes };

        match chunk.is_valid() {
            true => Ok(chunk),
            false => Err(Box::new(ChunkTypeError::InvalidCharacter)),
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
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(chunk.is_valid());

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
