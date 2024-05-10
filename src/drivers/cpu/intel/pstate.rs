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
    const NO_TURBO_FLAG: &'static str = "/sys/devices/system/cpu/intel_pstate/no_turbo";

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
}

#[async_trait]
impl crate::drivers::Driver for Driver {
    async fn activate(&self, power_profile: &crate::types::PowerProfile) -> Result<()> {
        if self.dry_run {
            log::debug!("Would have activated power profile {:#?}", power_profile);

            return Ok(());
        }

        Ok(())
    }

    async fn current(&self) -> Result<crate::types::InferredPowerProfile> {
        todo!()
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
