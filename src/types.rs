use serde::{Deserialize, Serialize};
use zvariant::Type;

#[derive(PartialEq)]
pub(crate) struct InferredPowerProfile {
    pub(crate) boost: bool,
    pub(crate) energy_preference: super::drivers::cpu::types::EnergyPreference,
    pub(crate) scaling_governor: super::drivers::cpu::types::ScalingGovernor,
    pub(crate) maximum_frequency: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct PowerProfile {
    pub(crate) cpu: crate::drivers::cpu::types::PowerProfile,
    #[serde(rename = "$key$")]
    pub(crate) name: String,
}

impl PartialEq<InferredPowerProfile> for PowerProfile {
    fn eq(&self, other: &InferredPowerProfile) -> bool {
        if self.cpu.boost != other.boost {
            return false;
        }

        if self.cpu.energy_preference != other.energy_preference {
            return false;
        }
        if self.cpu.scaling_governor != other.scaling_governor {
            return false;
        }

        true
    }
}

impl ToString for PowerProfile {
    fn to_string(&self) -> String {
        format!("PowerProfile(name={}, cpu={:#?})", self.name, self.cpu,)
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
