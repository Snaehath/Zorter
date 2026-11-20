use serde::{Deserialize, Serialize};
use crate::core::errors::Result;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedHeader {
    pub version: u32,
}

pub fn check_version(header_version: u32) -> Result<()> {
    if header_version > CURRENT_SCHEMA_VERSION {
        log::warn!(
            "Data schema version {} is newer than current supported version {}",
            header_version,
            CURRENT_SCHEMA_VERSION
        );
    }
    Ok(())
}
