use anyhow::{Context, Result};
use std::fs;
use std::str::FromStr;

use crate::types::{EnergyPreference, ScalingGovernor};

pub(crate) struct Driver {
    dry_run: bool,
    name: String,
    status: Status,
}

impl Driver {
    const ENERGY_PREFERENCE: &'static str =
        "/sys/devices/system/cpu/cpufreq/policy0/energy_performance_preference";
    const ONLINE_CPUS: &'static str = "/sys/devices/system/cpu/online";
    const BOOST_FLAG: &'static str = "/sys/devices/system/cpu/cpufreq/boost";
    const SCALING_GOVERNOR: &'static str =
        "/sys/devices/system/cpu/cpufreq/policy0/scaling_governor";

    pub fn new(dry_run: bool, name: String) -> Result<Self> {
        Ok(Self {
            dry_run: dry_run,
            name: name,
            status: Status::current()?,
        })
    }

    pub fn cores() -> Result<impl Iterator<Item = i32>> {
        let cores = String::from_str(fs::read_to_string(Self::ONLINE_CPUS).unwrap().trim())?;
        let mut range = cores.split("-");

        // TODO: probably doesn't handle all parse cases
        let first = range
            .next()
            .ok_or(anyhow::anyhow!("Failed to get beginning of range"))?
            .parse()?;
        let last: i32 = range
            .next()
            .ok_or(anyhow::anyhow!("Failed to get end of range"))?
            .parse()?;

        Ok(first..last)
    }

    fn boost_enabled(&self) -> Result<bool> {
        match fs::read_to_string(Self::BOOST_FLAG)
            .with_context(|| format!("Failed to read from {}", Self::BOOST_FLAG))
        {
            Ok(res) => Ok(res.trim().parse()?),
            Err(..) => Ok(true),
        }
    }

    fn energy_preference(&self) -> Result<EnergyPreference> {
        Ok(fs::read_to_string(Self::ENERGY_PREFERENCE)?
            .trim()
            .try_into()?)
    }

    fn scaling_governor(&self) -> Result<ScalingGovernor> {
        Ok(fs::read_to_string(Self::SCALING_GOVERNOR)?
            .trim()
            .try_into()?)
    }
}

impl crate::drivers::Driver for Driver {
    // TODO: Figure out a way to make this atomic
    fn activate(&self, power_profile: crate::types::PowerProfile) -> Result<()> {
        if self.dry_run {
            log::debug!(
                "Would have activated power profile {}",
                power_profile.to_string()
            );

            return Ok(());
        }

        if self.status.boost_supported() {
            log::debug!("Boost is supported!");

            if power_profile.boost {
                fs::write(Self::BOOST_FLAG, "1")?;
            }
        } else {
            log::warn!("Boost specified, but the current mode does not support it!");
        }

        log::debug!("Writing to /sys/devices/system/cpu/cpufreq/policy*/scaling_governor");

        Self::cores()?
            .map(|core_id| {
                Ok(fs::write::<&str, String>(
                    format!(
                        "/sys/devices/system/cpu/cpufreq/policy{:?}/scaling_governor",
                        core_id
                    )
                    .as_str(),
                    power_profile.scaling_governor.into(),
                )?)
            })
            .collect::<Result<()>>()?;

        log::debug!(
            "Writing to /sys/devices/system/cpu/cpufreq/policy*/energy_performance_preference"
        );

        Self::cores()?
            .map(|core_id| {
                Ok(fs::write::<&str, String>(
                    format!(
                        "/sys/devices/system/cpu/cpufreq/policy{:?}/energy_performance_preference",
                        core_id
                    )
                    .as_str(),
                    power_profile.energy_preference.into(),
                )?)
            })
            .collect()
    }

    fn current(&self) -> Result<crate::types::InferredPowerProfile> {
        Ok(crate::types::InferredPowerProfile {
            boost: self.boost_enabled()?,
            scaling_governor: self.scaling_governor()?,
            energy_preference: self.energy_preference()?,
        })
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

enum Status {
    Active,
    Guided,
    Passive,
}

impl Status {
    const PSTATE_STATUS_PATH: &'static str = "/sys/devices/system/cpu/amd_pstate/status";

    fn current() -> Result<Self> {
        Self::from_str(&fs::read_to_string(Self::PSTATE_STATUS_PATH)?.trim())
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
