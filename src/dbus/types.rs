// TODO: zvariant rename was not working...
#![allow(non_snake_case)]

use zvariant::{SerializeDict, Type};

use crate::types;

#[derive(Clone, Debug, SerializeDict, Type, zvariant::Value, zvariant::OwnedValue)]
#[zvariant(signature = "a{sv}", rename_all = "PascalCase")]
pub(crate) struct PowerProfile {
    Profile: String,
    CpuDriver: String,
    Driver: String,
    PlatformDriver: String,
}

impl PowerProfile {
    pub(crate) fn new(power_profile: &types::PowerProfile, driver: String) -> Self {
        Self {
            Profile: power_profile.name.clone(),
            CpuDriver: driver,
            PlatformDriver: "placeholder".to_string(),
            Driver: "multiple".to_string(),
        }
    }
}