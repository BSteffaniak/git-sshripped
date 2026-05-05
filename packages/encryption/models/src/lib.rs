#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use thiserror::Error;

pub const ENCRYPTED_MAGIC: [u8; 4] = *b"GSC1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EncryptionAlgorithm {
    /// Legacy AES-SIV format that binds ciphertext to the repository-relative path.
    AesSivV1,
    /// AES-SIV format with fixed associated data so encrypted files can move paths.
    AesSivMovableV1,
}

impl EncryptionAlgorithm {
    #[must_use]
    pub const fn id(self) -> u8 {
        match self {
            Self::AesSivV1 => 1,
            Self::AesSivMovableV1 => 2,
        }
    }

    /// Parse an algorithm from its binary id.
    ///
    /// # Errors
    ///
    /// Returns [`EncryptionModelsError::UnknownAlgorithm`] if `id` does not
    /// correspond to a known algorithm.
    pub const fn from_id(id: u8) -> Result<Self, EncryptionModelsError> {
        match id {
            1 => Ok(Self::AesSivV1),
            2 => Ok(Self::AesSivMovableV1),
            _ => Err(EncryptionModelsError::UnknownAlgorithm(id)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EncryptedHeader {
    pub version: u8,
    pub algorithm: EncryptionAlgorithm,
}

impl Default for EncryptedHeader {
    fn default() -> Self {
        Self {
            version: 1,
            algorithm: EncryptionAlgorithm::AesSivMovableV1,
        }
    }
}

#[derive(Debug, Error)]
pub enum EncryptionModelsError {
    #[error("unknown encryption algorithm id: {0}")]
    UnknownAlgorithm(u8),
    #[error("invalid encrypted file header")]
    InvalidHeader,
}
