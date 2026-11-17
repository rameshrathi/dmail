use crate::error::TypeError;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Opaque 32-byte identity identifier.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentityId([u8; 32]);

impl IdentityId {
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

    pub fn to_base64url(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.0)
    }

    pub fn from_base64url(s: &str) -> Result<Self, TypeError> {
        let bytes = URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|e| TypeError::InvalidBase64(e.to_string()))?;
        Self::from_slice(&bytes)
    }
}

impl fmt::Debug for IdentityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IdentityId({})", self.to_base64url())
    }
}

impl fmt::Display for IdentityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base64url())
    }
}

impl Serialize for IdentityId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_base64url())
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for IdentityId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Self::from_base64url(&s).map_err(serde::de::Error::custom)
        } else {
            let bytes = <Vec<u8>>::deserialize(deserializer)?;
            Self::from_slice(&bytes).map_err(serde::de::Error::custom)
        }
    }
}

/// Opaque 16-byte device identifier.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId([u8; 16]);

impl DeviceId {
    pub const BYTE_LEN: usize = 16;

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, TypeError> {
        if slice.len() != Self::BYTE_LEN {
            return Err(TypeError::InvalidLength {
                expected: Self::BYTE_LEN,
                actual: slice.len(),
            });
        }
        let mut arr = [0u8; 16];
        arr.copy_from_slice(slice);
        Ok(Self(arr))
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    pub fn to_base64url(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.0)
    }

    pub fn from_base64url(s: &str) -> Result<Self, TypeError> {
        let bytes = URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|e| TypeError::InvalidBase64(e.to_string()))?;
        Self::from_slice(&bytes)
    }
}

impl fmt::Debug for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DeviceId({})", self.to_base64url())
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base64url())
    }
}

impl Serialize for DeviceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_base64url())
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for DeviceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Self::from_base64url(&s).map_err(serde::de::Error::custom)
        } else {
            let bytes = <Vec<u8>>::deserialize(deserializer)?;
            Self::from_slice(&bytes).map_err(serde::de::Error::custom)
        }
    }
}

/// Key identifier string (1..=64 characters, safe alphanumeric, -, _).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyId(String);

impl KeyId {
    pub const MAX_LEN: usize = 64;

    pub fn new(s: impl Into<String>) -> Result<Self, TypeError> {
        let s = s.into();
        if s.is_empty() || s.len() > Self::MAX_LEN {
            return Err(TypeError::InvalidIdentifier(format!(
                "KeyId length must be 1..={}, got {}",
                Self::MAX_LEN,
                s.len()
            )));
        }
        if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err(TypeError::InvalidIdentifier(
                "KeyId must only contain ASCII alphanumeric, '-' or '_'".into(),
            ));
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KeyId(\"{}\")", self.0)
    }
}

impl fmt::Display for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for KeyId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for KeyId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

/// Message identifier (16 to 32 bytes).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageId(Vec<u8>);

impl MessageId {
    pub const MIN_LEN: usize = 16;
    pub const MAX_LEN: usize = 32;

    pub fn from_slice(slice: &[u8]) -> Result<Self, TypeError> {
        if slice.len() < Self::MIN_LEN || slice.len() > Self::MAX_LEN {
            return Err(TypeError::InvalidLength {
                expected: Self::MAX_LEN,
                actual: slice.len(),
            });
        }
        Ok(Self(slice.to_vec()))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn to_base64url(&self) -> String {
        URL_SAFE_NO_PAD.encode(&self.0)
    }

    pub fn from_base64url(s: &str) -> Result<Self, TypeError> {
        let bytes = URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|e| TypeError::InvalidBase64(e.to_string()))?;
        Self::from_slice(&bytes)
    }
}

impl fmt::Debug for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MessageId({})", self.to_base64url())
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base64url())
    }
}

impl Serialize for MessageId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_base64url())
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for MessageId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Self::from_base64url(&s).map_err(serde::de::Error::custom)
        } else {
            let bytes = <Vec<u8>>::deserialize(deserializer)?;
            Self::from_slice(&bytes).map_err(serde::de::Error::custom)
        }
    }
}

/// 32-byte cryptographic hash (SHA-256).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash32([u8; 32]);

impl Hash32 {
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

    pub fn to_base64url(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.0)
    }

    pub fn from_base64url(s: &str) -> Result<Self, TypeError> {
        let bytes = URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|e| TypeError::InvalidBase64(e.to_string()))?;
        Self::from_slice(&bytes)
    }
}

impl fmt::Debug for Hash32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash32({})", self.to_base64url())
    }
}

impl fmt::Display for Hash32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base64url())
    }
}

impl Serialize for Hash32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_base64url())
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for Hash32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Self::from_base64url(&s).map_err(serde::de::Error::custom)
        } else {
            let bytes = <Vec<u8>>::deserialize(deserializer)?;
            Self::from_slice(&bytes).map_err(serde::de::Error::custom)
        }
    }
}
