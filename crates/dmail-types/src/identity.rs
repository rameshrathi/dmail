use crate::error::TypeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Solana-compatible 32-byte wallet public key address.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WalletAddress([u8; 32]);

impl WalletAddress {
    pub const BYTE_LEN: usize = 32;

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, TypeError> {
        if slice.len() != Self::BYTE_LEN {
            return Err(TypeError::InvalidLength {
                expected: Self::BYTE_LEN,
                actual: slice.len(),
            });
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(slice);
        Ok(Self(arr))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_base58(&self) -> String {
        bs58::encode(self.0).into_string()
    }

    pub fn from_base58(s: &str) -> Result<Self, TypeError> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|e| TypeError::Base58Error(e.to_string()))?;
        Self::from_slice(&bytes)
    }
}

impl fmt::Debug for WalletAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WalletAddress({})", self.to_base58())
    }
}

impl fmt::Display for WalletAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base58())
    }
}

impl Serialize for WalletAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_base58())
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for WalletAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Self::from_base58(&s).map_err(serde::de::Error::custom)
        } else {
            let bytes = <Vec<u8>>::deserialize(deserializer)?;
            Self::from_slice(&bytes).map_err(serde::de::Error::custom)
        }
    }
}

/// 32-byte X25519 Device Public Encryption Key.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EncryptionPublicKey([u8; 32]);

impl EncryptionPublicKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_base58(&self) -> String {
        bs58::encode(self.0).into_string()
    }

    pub fn from_base58(s: &str) -> Result<Self, TypeError> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|e| TypeError::Base58Error(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(TypeError::InvalidLength {
                expected: 32,
                actual: bytes.len(),
            });
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }
}

impl fmt::Debug for EncryptionPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EncryptionPublicKey({})", self.to_base58())
    }
}

impl fmt::Display for EncryptionPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base58())
    }
}

impl Serialize for EncryptionPublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_base58())
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for EncryptionPublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Self::from_base58(&s).map_err(serde::de::Error::custom)
        } else {
            let bytes = <Vec<u8>>::deserialize(deserializer)?;
            if bytes.len() != 32 {
                return Err(serde::de::Error::custom("Expected 32 bytes"));
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            Ok(Self(arr))
        }
    }
}

/// 64-byte Ed25519 digital signature wrapper with Serialize and Deserialize support.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Signature64([u8; 64]);

impl Signature64 {
    pub fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

impl fmt::Debug for Signature64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature64({} bytes)", self.0.len())
    }
}

impl Serialize for Signature64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let hex = self
                .0
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
            serializer.serialize_str(&hex)
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for Signature64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            if s.len() != 128 {
                return Err(serde::de::Error::custom(
                    "Expected 128-char hex string for signature",
                ));
            }
            let mut bytes = [0u8; 64];
            for i in 0..64 {
                bytes[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
                    .map_err(serde::de::Error::custom)?;
            }
            Ok(Self(bytes))
        } else {
            let bytes = <Vec<u8>>::deserialize(deserializer)?;
            if bytes.len() != 64 {
                return Err(serde::de::Error::custom("Expected 64 bytes for signature"));
            }
            let mut arr = [0u8; 64];
            arr.copy_from_slice(&bytes);
            Ok(Self(arr))
        }
    }
}

/// On-chain registry record mapping a wallet address to its verified public encryption key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserKeyRecord {
    pub wallet_address: WalletAddress,
    pub encryption_pubkey: EncryptionPublicKey,
    pub registered_at: i64,
    pub key_version: u32,
    pub is_active: bool,
}
