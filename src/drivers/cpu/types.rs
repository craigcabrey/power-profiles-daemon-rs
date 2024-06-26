use std::str::FromStr;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(PartialEq)]
pub(crate) struct InferredPowerProfile {
    pub(crate) boost: bool,
    pub(crate) energy_preference: EnergyPreference,
    pub(crate) scaling_governor: ScalingGovernor,
    pub(crate) maximum_frequency: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PowerProfile {
    pub(crate) name: Option<String>,
    pub(crate) boost: bool,
    pub(crate) energy_preference: EnergyPreference,
    pub(crate) scaling_governor: ScalingGovernor,
    pub(crate) maximum_frequency: Option<u32>,
    pub(crate) driver_options: Option<config::Value>,
}

impl From<crate::types::PowerProfile> for PowerProfile {
    fn from(value: crate::types::PowerProfile) -> Self {
        Self {
            name: Some(value.name.clone()),
            ..value.cpu.to_owned()
        }
    }
}

// TODO: serde doesn't recognize snake_case from the config json...
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
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

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
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
