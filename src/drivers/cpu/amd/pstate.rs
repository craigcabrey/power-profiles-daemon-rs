use anyhow::{Context, Result};
use async_std::fs;
use async_trait::async_trait;
use serde::Deserialize;
use std::{collections::HashMap, str::FromStr};

use super::super::types::{EnergyPreference, ScalingGovernor};
use crate::drivers::cpu::utils;

#[derive(Deserialize)]
pub(crate) struct DriverSettings {}

pub(crate) struct Driver {
    dry_run: bool,
    name: String,
    status: Status,
    _profile_driver_settings: HashMap<String, DriverSettings>,
}

impl Driver {
    const ENERGY_PREFERENCE: &'static str =
        "/sys/devices/system/cpu/cpufreq/policy0/energy_performance_preference";
    const BOOST_FLAG: &'static str = "/sys/devices/system/cpu/cpufreq/boost";
    const SCALING_GOVERNOR: &'static str =
        "/sys/devices/system/cpu/cpufreq/policy0/scaling_governor";

    pub async fn new(
        dry_run: bool,
        name: String,
        profile_driver_settings: HashMap<String, DriverSettings>,
    ) -> Result<Self> {
        Ok(Self {
            dry_run: dry_run,
            name: name,
            status: Status::current().await?,
            _profile_driver_settings: profile_driver_settings,
        })
    }

    async fn boost_enabled(&self) -> Result<bool> {
        match fs::read_to_string(Self::BOOST_FLAG)
            .await
            .with_context(|| format!("Failed to read from {}", Self::BOOST_FLAG))
        {
            Ok(res) => Ok(res.trim().parse()?),
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
    // TODO: Figure out a way to make this atomic
    async fn activate(&self, power_profile: &super::super::types::PowerProfile) -> Result<()> {
        log::debug!("Activating profile {:?}", power_profile);

        if self.dry_run {
            log::debug!("Would have activated power profile {:#?}", power_profile);

            return Ok(());
        }

        if self.status.boost_supported() {
            log::debug!("Boost is supported!");

            if power_profile.boost {
                fs::write(Self::BOOST_FLAG, "1").await?;
            }
        } else {
            log::warn!("Boost specified, but the current mode does not support it!");
        }

        utils::activate_maximum_frequency(power_profile.maximum_frequency).await?;
        utils::activate_scaling_governor(power_profile.scaling_governor).await?;
        utils::activate_energy_preference(power_profile.energy_preference).await?;

        Ok(())
    }

    async fn current(&self) -> Result<crate::types::InferredPowerProfile> {
        Ok(crate::types::InferredPowerProfile {
            boost: self.boost_enabled().await?,
            scaling_governor: self.scaling_governor().await?,
            energy_preference: self.energy_preference().await?,
            maximum_frequency: utils::maximum_frequency().await?,
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

enum Status {
    Active,
    Guided,
    Passive,
}

impl Status {
    const PSTATE_STATUS_PATH: &'static str = "/sys/devices/system/cpu/amd_pstate/status";

    async fn current() -> Result<Self> {
        Self::from_str(&fs::read_to_string(Self::PSTATE_STATUS_PATH).await?.trim())
    }

    fn boost_supported(&self) -> bool {
        match self {
            Self::Active => false,
            Self::Guided => true,
            Self::Passive => true,
        }
    }
}

impl FromStr for Status {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "active" => Ok(Status::Active),
            "guided" => Ok(Status::Guided),
            "passive" => Ok(Status::Passive),
            _ => Err(anyhow::anyhow!(
                "Unrecognized amd-pstate driver status {}",
                s
            )),
        }
    }
}
