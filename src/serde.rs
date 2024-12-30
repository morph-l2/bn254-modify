use std::io::{self, Read, Write};

/// Trait for converting raw bytes to/from the internal representation of a type.
/// For example, field elements are represented in Montgomery form and serialized/deserialized without Montgomery reduction.
pub trait SerdeObject: Sized {
    /// The purpose of unchecked functions is to read the internal memory representation
    /// of a type from bytes as quickly as possible. No sanitization checks are performed
    /// to ensure the bytes represent a valid object. As such this function should only be
    /// used internally as an extension of machine memory. It should not be used to deserialize
    /// externally provided data.
    fn from_raw_bytes_unchecked(bytes: &[u8]) -> Self;
    fn from_raw_bytes(bytes: &[u8]) -> Option<Self>;

    fn to_raw_bytes(&self) -> Vec<u8>;

    /// The purpose of unchecked functions is to read the internal memory representation
    /// of a type from disk as quickly as possible. No sanitization checks are performed
    /// to ensure the bytes represent a valid object. This function should only be used
    /// internally when some machine state cannot be kept in memory (e.g., between runs)
    /// and needs to be reloaded as quickly as possible.
    fn read_raw_unchecked<R: Read>(reader: &mut R) -> Self;
    fn read_raw<R: Read>(reader: &mut R) -> io::Result<Self>;

    fn write_raw<W: Write>(&self, writer: &mut W) -> io::Result<()>;
}

impl SerdeObject for crate::Fr {
    fn from_raw_bytes_unchecked(bytes: &[u8]) -> Self {
        let mut tmp = [0u64; 4];
        let chunks = bytes.chunks_exact(8);
        for (i, chunk) in chunks.take(4).enumerate() {
            tmp[i] = u64::from_le_bytes(chunk.try_into().unwrap());
        }
        Self(tmp)
    }

    fn from_raw_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 32 {
            return None;
        }
        Some(Self::from_raw_bytes_unchecked(bytes))
    }

    fn to_raw_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(32);
        for limb in self.0.iter() {
            bytes.extend_from_slice(&limb.to_le_bytes());
        }
        bytes
    }

    fn read_raw_unchecked<R: Read>(reader: &mut R) -> Self {
        let mut bytes = [0u8; 32];
        reader.read_exact(&mut bytes).unwrap();
        Self::from_raw_bytes_unchecked(&bytes)
    }

    fn read_raw<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut bytes = [0u8; 32];
        reader.read_exact(&mut bytes)?;
        Ok(Self::from_raw_bytes_unchecked(&bytes))
    }

    fn write_raw<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.to_raw_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Fr;

    #[test]
    fn test_serde_roundtrip() {
        let fr = Fr::from_raw([1, 2, 3, 4]);
        let bytes = fr.to_raw_bytes();
        let fr2 = Fr::from_raw_bytes(&bytes).unwrap();
        assert_eq!(fr, fr2);
    }

    #[test]
    fn test_invalid_bytes() {
        assert!(Fr::from_raw_bytes(&[0; 31]).is_none());
    }
}