// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level LUKS2 configuration structure representing a LUKS2 encrypted device.
/// This structure contains all the configuration needed to manage a LUKS2 device,
/// including core configuration, keyslots and segments.
#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2Config {
    /// Core configuration data containing metadata and keyslot sizes
    pub config: Luks2ConfigData,
    /// Map of keyslot IDs to their corresponding keyslot configurations.
    /// Each keyslot contains information about key derivation and storage.
    pub keyslots: HashMap<u64, Luks2Keyslot>,
    /// Map of segment IDs to their corresponding segment configurations.
    /// Segments define the encrypted regions of the device.
    pub segments: HashMap<u64, Luks2Segment>,
    // pub tokens: HashMap<u64, Value>,
    // pub digests: HashMap<u64, Value>,
}

/// Core LUKS2 configuration data containing essential metadata about the encrypted device.
#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2ConfigData {
    /// Size of the JSON metadata area in bytes.
    /// This defines how much space is reserved for the LUKS2 header.
    #[serde(with = "display_from_str")]
    pub json_size: u64,

    /// Size of the keyslots area in bytes.
    /// This defines the total space available for storing encrypted keys.
    #[serde(with = "display_from_str")]
    pub keyslots_size: u64,

    /// Optional configuration flags that modify device behavior.
    /// These flags can affect how the device is activated and used.
    #[serde(default)]
    pub flags: Vec<String>,

    /// Requirements for device activation.
    /// These define what conditions must be met to unlock the device.
    #[serde(default)]
    pub requirements: Vec<String>,
}

/// Key derivation function (KDF) configuration used to generate encryption keys from passwords.
#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2Kdf {
    /// Type of KDF (e.g. pbkdf2, argon2i, argon2id)
    /// Specifies which algorithm is used for key derivation
    #[serde(rename = "type")]
    pub kdf_type: String,
    /// Random salt used in key derivation to prevent rainbow table attacks
    pub salt: String,

    /// Hash algorithm used for PBKDF2
    /// Only applicable when kdf_type is "pbkdf2"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Number of iterations for PBKDF2
    /// Only applicable when kdf_type is "pbkdf2"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iterations: Option<u64>,

    /// Time cost parameter for Argon2
    /// Only applicable when kdf_type is "argon2i" or "argon2id"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<u64>,
    /// Memory usage in bytes for Argon2
    /// Only applicable when kdf_type is "argon2i" or "argon2id"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<u64>,
    /// Number of parallel threads for Argon2
    /// Only applicable when kdf_type is "argon2i" or "argon2id"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus: Option<u64>,
}

/// Configuration for a single keyslot containing key material and derivation settings.
#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2Keyslot {
    /// Type of keyslot, defining how the key material is processed
    #[serde(rename = "type")]
    pub slot_type: String,

    /// Size of the keyslot key in bytes
    pub key_size: u64,

    /// Storage area configuration defining where and how key material is stored
    pub area: Luks2KeyslotArea,
    /// Key derivation parameters used to process passwords into keys
    pub kdf: Luks2Kdf,
}

/// Configuration for keyslot storage area defining where encrypted keys are stored.
#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2KeyslotArea {
    /// Type of storage area, defining how the area is organized
    #[serde(rename = "type")]
    pub area_type: String,

    /// Offset in bytes where this area begins on the device
    #[serde(with = "display_from_str")]
    pub offset: u64,

    /// Size of this area in bytes
    #[serde(with = "display_from_str")]
    pub size: u64,

    /// Encryption algorithm used to protect stored key material
    pub encryption: String,

    /// Size of encryption key in bytes used for this area
    pub key_size: u64,
}

/// Configuration for a disk segment defining an encrypted region of the device.
#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2Segment {
    /// Type of segment, defining how the region is processed
    #[serde(rename = "type")]
    pub segment_type: String,
    /// Offset where segment begins, as a string representation
    pub offset: String,
    /// Size of segment, as a string representation
    pub size: String,
    /// Initialization vector tweak used for encryption
    pub iv_tweak: String,
    /// Encryption algorithm used for this segment
    pub encryption: String,
    /// Sector size in bytes - the granularity of encryption
    pub sector_size: u64,
}

mod display_from_str {
    use std::{fmt::Display, str::FromStr};

    use serde::{Deserialize, Deserializer, Serializer};

    pub(super) fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub(super) fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}
