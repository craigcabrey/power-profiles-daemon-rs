use anyhow::{Context, Result};
use async_std::fs;
use async_trait::async_trait;
use std::str::FromStr;

use crate::drivers::cpu::utils;

use super::super::types::{EnergyPreference, ScalingGovernor};

pub(crate) struct Driver {
    dry_run: bool,
    _status: Status,
}

impl Driver {
    const ENERGY_PREFERENCE: &'static str =
        "/sys/devices/system/cpu/cpufreq/policy0/energy_performance_preference";
    const NO_TURBO_FLAG: &'static str = "/sys/devices/system/cpu/intel_pstate/no_turbo";
    const SCALING_GOVERNOR: &'static str =
        "/sys/devices/system/cpu/cpufreq/policy0/scaling_governor";

    pub async fn new(dry_run: bool) -> Result<Self> {
        Ok(Self {
            dry_run: dry_run,
            _status: Status::current().await?,
        })
    }

    async fn turbo_enabled(&self) -> Result<bool> {
        match fs::read_to_string(Self::NO_TURBO_FLAG)
            .await
            .with_context(|| format!("Failed to read from {}", Self::NO_TURBO_FLAG))
        {
            Ok(res) => Ok(!res.trim().parse()?),
            Err(..) => Ok(true),
        }
    }

    async fn energy_preference(&self) -> Result<EnergyPreference> {
        Ok(fs::read_to_string(Self::ENERGY_PREFERENCE)
            .await?
            .trim()
            .try_into()?)
    }

    async fn scaling_governor(&self) -> Result<ScalingGovernor> {
        Ok(fs::read_to_string(Self::SCALING_GOVERNOR)
            .await?
            .trim()
            .try_into()?)
    }
}

#[async_trait]
impl crate::drivers::Driver for Driver {
    async fn activate(&self, power_profile: &super::super::types::PowerProfile) -> Result<()> {
        if self.dry_run {
            log::debug!("Would have activated power profile {:#?}", power_profile);

            return Ok(());
        }

        utils::activate_maximum_frequency(power_profile.maximum_frequency).await?;
        utils::activate_scaling_governor(power_profile.scaling_governor).await?;
        utils::activate_energy_preference(power_profile.energy_preference).await?;

        Ok(())
    }

    async fn current(&self) -> Result<crate::types::InferredPowerProfile> {
        Ok(crate::types::InferredPowerProfile {
            boost: self.turbo_enabled().await?,
            scaling_governor: self.scaling_governor().await?,
            energy_preference: self.energy_preference().await?,
            maximum_frequency: utils::maximum_frequency().await?,
        })
    }

    fn name(&self) -> &str {
        "intel_pstate"
    }
}

enum Status {
    Active,
    Off,
    Passive,
}

impl Status {
    const PSTATE_STATUS_PATH: &'static str = "/sys/devices/system/cpu/intel_pstate/status";

    async fn current() -> Result<Self> {
        Self::from_str(&fs::read_to_string(Self::PSTATE_STATUS_PATH).await?.trim())
    }
}

impl FromStr for Status {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "active" => Ok(Status::Active),
            "off" => Ok(Status::Off),
            "passive" => Ok(Status::Passive),
            _ => Err(anyhow::anyhow!(
                "Unrecognized intel-pstate driver status {}",
                s
            )),
        }
    }
}
