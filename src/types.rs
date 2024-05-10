use serde::{Deserialize, Serialize};
use zvariant::Type;

use crate::drivers::cpu;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct PowerProfile {
    pub(crate) cpu: cpu::types::PowerProfile,
    #[serde(rename = "$key$")]
    pub(crate) name: String,
}

impl ToString for PowerProfile {
    fn to_string(&self) -> String {
        format!("PowerProfile(name={}, cpu={:#?})", self.name, self.cpu,)
    }
}

#[derive(PartialEq)]
pub(crate) struct InferredPowerProfile {
    pub(crate) cpu: Option<cpu::types::InferredPowerProfile>,
}

impl PartialEq<PowerProfile> for InferredPowerProfile {
    fn eq(&self, other: &PowerProfile) -> bool {
        let cpu_profile = self.cpu.as_ref().unwrap();
        //other.cpu
        let other_cpu_profile = &other.cpu;

        cpu_profile == other_cpu_profile
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Type, zvariant::Value, zvariant::OwnedValue)]
#[zvariant(signature = "dict")]
pub(crate) struct PowerProfileHold {
    #[zvariant(rename = "ApplicationId")]
    pub(crate) application_id: String,
    #[zvariant(rename = "Profile")]
    pub(crate) profile: String,
    #[zvariant(rename = "Reason")]
    pub(crate) reason: String,
}

impl PowerProfileHold {
    pub fn new(application_id: String, profile: String, reason: String) -> Self {
        PowerProfileHold {
            application_id: application_id,
            profile: profile,
            reason: reason,
        }
    }
}
