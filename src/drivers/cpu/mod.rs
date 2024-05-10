#![allow(unused)]

use std::sync::Arc;

use anyhow::Result;
use async_std::fs;
use async_trait::async_trait;

use self::types::PowerProfile;

mod amd;
mod dummy;
mod intel;
pub(crate) mod types;
mod utils;

use super::cpu::types::{EnergyPreference, ScalingGovernor};
use crate::drivers;

pub async fn probe(
    profiles: &Vec<PowerProfile>,
) -> Result<Arc<dyn crate::drivers::Driver + Send + Sync>> {
    let driver = Driver::from_system().await?;
    log::trace!("Loaded {:#?}", driver);
    Ok(Arc::new(driver))
}

#[derive(Debug)]
pub(crate) struct Driver {
    dry_run: bool,
    platform_driver: Option<Box<dyn crate::drivers::Driver>>,
    policies: Vec<Policy>,
}

impl Driver {
    const ONLINE_CPUS: &'static str = "/sys/devices/system/cpu/online";

    pub(crate) async fn from_system() -> Result<Self> {
        Ok(Self {
            // FIXME
            dry_run: false,
            platform_driver: None,
            policies: futures::future::join_all(
                Self::online_cpu_id_iter(&Self::online_cpus().await?)?
                    .map(|cpu_id| Policy::from_cpu_id(cpu_id)),
            )
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?,
        })
    }

    async fn online_cpus() -> Result<String, std::io::Error> {
        fs::read_to_string(Self::ONLINE_CPUS).await
    }

    fn online_cpu_id_iter(online_cpus: &String) -> Result<impl Iterator<Item = u32> + '_> {
        Ok(online_cpus
            .trim()
            // "1-5,7-9" -> ["1-5", "7-9"]
            .split(",")
            // ["1-5", "7-9", "hello-world", "rust"] -> [["1","5"], ["7","9"], ["hello","world"]]
            .filter_map(|token| token.split_once("-"))
            // [["1","5"], ["7","9"], ["hello","world"]] -> [[1,5], [7,9]]]
            .filter_map(|(first, second)| first.parse::<u32>().ok().zip(second.parse::<u32>().ok()))
            .flat_map(|(first, second)| (first..=second).into_iter()))
    }
}

#[async_trait]
impl crate::drivers::Driver for Driver {
    async fn activate(&self, power_profile: &crate::types::PowerProfile) -> Result<()> {
        log::trace!("Activating profile {:#?}", power_profile);

        if self.dry_run {
            log::debug!("Would have activated power profile {:#?}", power_profile);

            return Ok(());
        }

        futures::future::join_all(
            self.policies
                .iter()
                .map(|policy| policy.activate(power_profile.cpu.cpu_power_profile(policy.cpu_id))),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>();

        Ok(())
    }

    async fn current(&self) -> Result<crate::types::InferredPowerProfile> {
        Ok(crate::types::InferredPowerProfile {
            boost: true,
            energy_preference: EnergyPreference::Performance,
            maximum_frequency: 4000000,
            scaling_governor: ScalingGovernor::Performance,
        })
    }

    fn name(&self) -> &str {
        "cpufreq"
    }
}

#[derive(Debug)]
struct Policy {
    affected_cpus: Vec<u32>,
    cpu_id: u32,
    cpuinfo_min_freq: u32,
    cpuinfo_max_freq: u32,
    cpuinfo_transition_latency: u32,
    energy_performance_available_preferences: Vec<String>,
    energy_performance_preference: String,
    related_cpus: Vec<u32>,
    scaling_available_governors: Vec<String>,
    scaling_driver: String,
    scaling_cur_freq: u32,
    scaling_governor: String,
    scaling_min_freq: u32,
    scaling_max_freq: u32,
}

impl Policy {
    const AFFECTED_CPUS: &'static str = "affected_cpus";
    const CPUINFO_MAX_FREQ: &'static str = "cpuinfo_max_freq";
    const CPUINFO_MIN_FREQ: &'static str = "cpuinfo_min_freq";
    const CPUINFO_TRANSITION_LATENCY: &'static str = "cpuinfo_transition_latency";
    const ENERGY_PERFORMANCE_AVAILABLE_PREFERENCES: &'static str =
        "energy_performance_available_preferences";
    const ENERGY_PERFORMANCE_PREFERENCE: &'static str = "energy_performance_preference";
    const RELATED_CPUS: &'static str = "related_cpus";
    const SCALING_AVAILABLE_GOVERNORS: &'static str = "scaling_available_governors";
    const SCALING_DRIVER: &'static str = "scaling_driver";
    const SCALING_CUR_FREQ: &'static str = "scaling_cur_freq";
    const SCALING_GOVERNOR: &'static str = "scaling_governor";
    const SCALING_MIN_FREQ: &'static str = "scaling_min_freq";
    const SCALING_MAX_FREQ: &'static str = "scaling_max_freq";
    const SCALING_SETSPEED: &'static str = "scaling_setspeed";

    pub(crate) async fn from_cpu_id(cpu_id: u32) -> Result<Self> {
        Ok(Self {
            affected_cpus: Self::read_policy_property(cpu_id, Self::AFFECTED_CPUS)
                .await?
                .split(" ")
                .map(|s| s.parse().unwrap())
                .collect::<Vec<u32>>(),
            cpu_id: cpu_id,
            cpuinfo_max_freq: Self::read_policy_property(cpu_id, Self::CPUINFO_MAX_FREQ)
                .await?
                .parse()?,
            cpuinfo_min_freq: Self::read_policy_property(cpu_id, Self::CPUINFO_MIN_FREQ)
                .await?
                .parse()?,
            cpuinfo_transition_latency: Self::read_policy_property(
                cpu_id,
                Self::CPUINFO_TRANSITION_LATENCY,
            )
            .await?
            .parse()?,
            energy_performance_available_preferences: Self::read_policy_property(
                cpu_id,
                Self::ENERGY_PERFORMANCE_AVAILABLE_PREFERENCES,
            )
            .await?
            .split(" ")
            .map(|item| item.to_string())
            .collect(),
            energy_performance_preference: Self::read_policy_property(
                cpu_id,
                Self::ENERGY_PERFORMANCE_PREFERENCE,
            )
            .await?
            .parse()?,
            related_cpus: Self::read_policy_property(cpu_id, Self::RELATED_CPUS)
                .await?
                .split(" ")
                .map(|s| s.parse().unwrap())
                .collect::<Vec<u32>>(),
            scaling_available_governors: Self::read_policy_property(
                cpu_id,
                Self::SCALING_AVAILABLE_GOVERNORS,
            )
            .await?
            .to_owned()
            .split(" ")
            .map(|item| item.to_string())
            .collect(),
            scaling_driver: Self::read_policy_property(cpu_id, Self::SCALING_DRIVER).await?,
            scaling_cur_freq: Self::read_policy_property(cpu_id, Self::SCALING_CUR_FREQ)
                .await?
                .parse()?,
            scaling_governor: Self::read_policy_property(cpu_id, Self::SCALING_GOVERNOR).await?,
            scaling_min_freq: Self::read_policy_property(cpu_id, Self::SCALING_MIN_FREQ)
                .await?
                .parse()?,
            scaling_max_freq: Self::read_policy_property(cpu_id, Self::SCALING_MAX_FREQ)
                .await?
                .parse()?,
        })
    }

    pub(crate) async fn activate(&self, core_power_profile: &types::CpuPowerProfile) -> Result<()> {
        // Scaling governor must be written first
        Self::write_policy_property(
            self.cpu_id,
            Self::SCALING_GOVERNOR,
            core_power_profile.scaling_governor.into(),
        )
        .await?;
        Self::write_policy_property(
            self.cpu_id,
            Self::ENERGY_PERFORMANCE_PREFERENCE,
            core_power_profile.energy_preference.into(),
        )
        .await
    }

    async fn read_policy_property(cpu_id: u32, property: &str) -> Result<String> {
        Ok(fs::read_to_string(format!(
            "/sys/devices/system/cpu/cpufreq/policy{}/{}",
            cpu_id, property
        ))
        .await?
        .trim()
        .to_owned())
    }

    async fn scaling_setspeed(&self, value: u32) -> Result<()> {
        Self::write_policy_property(self.cpu_id, Self::SCALING_SETSPEED, value.to_string()).await
    }

    async fn write_policy_property(cpu_id: u32, property: &str, value: String) -> Result<()> {
        log::trace!(
            "Writing {} to /sys/devices/system/cpu/cpufreq/policy{}/{}",
            value,
            cpu_id,
            property,
        );

        Ok(fs::write(
            format!(
                "/sys/devices/system/cpu/cpufreq/policy{}/{}",
                cpu_id, property
            ),
            value,
        )
        .await?)
    }
}
