use std::str::FromStr;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use zvariant::Type;

#[derive(PartialEq)]
pub(crate) struct InferredPowerProfile {
    pub(crate) boost: bool,
    pub(crate) energy_preference: EnergyPreference,
    pub(crate) scaling_governor: ScalingGovernor,
}

#[derive(Clone, Debug, Deserialize, Serialize, zvariant::Value, zvariant::OwnedValue, Type)]
pub(crate) struct PowerProfile {
    pub(crate) boost: bool,
    pub(crate) energy_preference: EnergyPreference,
    #[serde(rename = "$key$")]
    pub(crate) name: String,
    pub(crate) scaling_governor: ScalingGovernor,
}

impl PowerProfile {
    pub fn _new(
        boost: bool,
        energy_preference: EnergyPreference,
        name: String,
        scaling_governor: ScalingGovernor,
    ) -> Self {
        PowerProfile {
            boost: boost,
            energy_preference: energy_preference,
            name: name,
            scaling_governor: scaling_governor,
        }
    }
}

impl PartialEq<InferredPowerProfile> for PowerProfile {
    fn eq(&self, other: &InferredPowerProfile) -> bool {
        if self.boost != other.boost {
            return false;
        }

        if self.energy_preference != other.energy_preference {
            return false;
        }
        if self.scaling_governor != other.scaling_governor {
            return false;
        }

        true
    }
}

impl ToString for PowerProfile {
    fn to_string(&self) -> String {
        format!(
            "PowerProfile(name={}, boost={}, energy_preference={}, scaling_governor={})",
            self.name,
            self.boost,
            self.energy_preference.to_string(),
            self.scaling_governor.to_string(),
        )
    }
}

// TODO: serde doesn't recognize snake_case from the config json...
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    PartialEq,
    Serialize,
    zvariant::Value,
    zvariant::OwnedValue,
    Type,
)]
pub(crate) enum EnergyPreference {
    #[serde(alias = "default")]
    Default = 0,

    #[serde(alias = "performance")]
    Performance = 1,

    #[serde(alias = "balance_performance")]
    BalancePerformance = 2,

    #[serde(alias = "balancePower")]
    BalancePower = 3,

    #[serde(alias = "power")]
    Power = 4,
}

impl ToString for EnergyPreference {
    fn to_string(&self) -> String {
        match self {
            Self::Default => "default",
            Self::Performance => "performance",
            Self::BalancePerformance => "balance_performance",
            Self::BalancePower => "balance_power",
            Self::Power => "power",
        }
        .to_string()
    }
}

impl Into<String> for EnergyPreference {
    fn into(self) -> String {
        self.to_string()
    }
}

impl TryFrom<&str> for EnergyPreference {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> anyhow::Result<Self> {
        Self::from_str(value)
    }
}

impl FromStr for EnergyPreference {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<EnergyPreference> {
        match s {
            "default" => Ok(Self::Default),
            "performance" => Ok(Self::Performance),
            "balance_performance" => Ok(Self::BalancePerformance),
            "balance_power" => Ok(Self::BalancePower),
            "power" => Ok(Self::Power),
            _ => Err(anyhow::anyhow!("No such energy preference {}", s)),
        }
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    PartialEq,
    Serialize,
    zvariant::Value,
    zvariant::OwnedValue,
    Type,
)]
pub(crate) enum ScalingGovernor {
    Performance = 0,
    Powersave = 1,
}

impl ToString for ScalingGovernor {
    fn to_string(&self) -> String {
        match self {
            Self::Performance => "performance",
            Self::Powersave => "powersave",
        }
        .to_string()
    }
}

impl TryFrom<&str> for ScalingGovernor {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> anyhow::Result<Self> {
        Self::from_str(value)
    }
}

impl FromStr for ScalingGovernor {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "performance" => Ok(ScalingGovernor::Performance),
            "powersave" => Ok(ScalingGovernor::Powersave),
            _ => Err(anyhow::anyhow!("No conversion possible from {}", s)),
        }
    }
}

impl Into<String> for ScalingGovernor {
    fn into(self) -> String {
        self.to_string()
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
