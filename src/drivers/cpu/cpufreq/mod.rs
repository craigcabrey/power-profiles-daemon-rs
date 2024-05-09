#![allow(unused)]

use std::sync::Arc;

use anyhow::Result;
use async_std::fs;
use async_trait::async_trait;

use crate::drivers::cpu::utils;

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

use crate::drivers;

pub(crate) const DRIVER: drivers::DriverModule = drivers::DriverModule {
    name: "cpufreq",
    probe: probe,
};

#[derive(Debug)]
pub(crate) struct Driver {
    policies: Vec<Policy>,
}

#[async_trait]
impl crate::drivers::Driver for Driver {
    async fn activate(&self, _power_profile: crate::types::PowerProfile) -> Result<()> {
        log::debug!("Activating!");
        Ok(())
    }

    async fn current(&self) -> Result<crate::types::InferredPowerProfile> {
        Ok(crate::types::InferredPowerProfile {
            boost: true,
            energy_preference: crate::types::EnergyPreference::Performance,
            maximum_frequency: 4000000,
            scaling_governor: crate::types::ScalingGovernor::Performance,
        })
    }

    fn name(&self) -> String {
        "dummy".to_string()
    }
}

impl Driver {
    pub(crate) async fn from_system() -> Result<Self> {
        Ok(Self {
            policies: futures::future::join_all(
                utils::online_cpu_id_iter(&utils::online_cpus().await?)?
                    .map(|cpu_id| Policy::from_cpu_id(cpu_id)),
            )
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

pub fn probe() -> Result<Arc<dyn crate::drivers::Driver + Send + Sync>> {
    Ok(futures::executor::block_on(aprobe())?)
}

pub async fn aprobe() -> Result<Arc<dyn crate::drivers::Driver + Send + Sync>> {
    let driver = Driver::from_system().await?;
    log::trace!("Loaded {:#?}", driver);
    Ok(Arc::new(driver))
}

#[derive(Debug)]
pub(crate) struct CPUFreq {
    policies: Vec<Policy>,
}

impl CPUFreq {
    pub(crate) async fn from_system() -> Result<Self> {
        Ok(Self {
            policies: futures::future::join_all(
                utils::online_cpu_id_iter(&utils::online_cpus().await?)?
                    .map(|cpu_id| Policy::from_cpu_id(cpu_id)),
            )
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?,
        })
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
    pub(crate) async fn from_cpu_id(cpu_id: u32) -> Result<Self> {
        Ok(Self {
            affected_cpus: Self::read_policy_property(cpu_id, AFFECTED_CPUS)
                .await?
                .split(" ")
                .map(|s| s.parse().unwrap())
                .collect::<Vec<u32>>(),
            cpu_id: cpu_id,
            cpuinfo_max_freq: Self::read_policy_property(cpu_id, CPUINFO_MAX_FREQ)
                .await?
                .parse()?,
            cpuinfo_min_freq: Self::read_policy_property(cpu_id, CPUINFO_MIN_FREQ)
                .await?
                .parse()?,
            cpuinfo_transition_latency: Self::read_policy_property(
                cpu_id,
                CPUINFO_TRANSITION_LATENCY,
            )
            .await?
            .parse()?,
            energy_performance_available_preferences: Self::read_policy_property(
                cpu_id,
                ENERGY_PERFORMANCE_AVAILABLE_PREFERENCES,
            )
            .await?
            .split(" ")
            .map(|item| item.to_string())
            .collect(),
            energy_performance_preference: Self::read_policy_property(
                cpu_id,
                ENERGY_PERFORMANCE_PREFERENCE,
            )
            .await?
            .parse()?,
            related_cpus: Self::read_policy_property(cpu_id, RELATED_CPUS)
                .await?
                .split(" ")
                .map(|s| s.parse().unwrap())
                .collect::<Vec<u32>>(),
            scaling_available_governors: Self::read_policy_property(
                cpu_id,
                SCALING_AVAILABLE_GOVERNORS,
            )
            .await?
            .to_owned()
            .split(" ")
            .map(|item| item.to_string())
            .collect(),
            scaling_driver: Self::read_policy_property(cpu_id, SCALING_DRIVER).await?,
            scaling_cur_freq: Self::read_policy_property(cpu_id, SCALING_CUR_FREQ)
                .await?
                .parse()?,
            scaling_governor: Self::read_policy_property(cpu_id, SCALING_GOVERNOR).await?,
            scaling_min_freq: Self::read_policy_property(cpu_id, SCALING_MIN_FREQ)
                .await?
                .parse()?,
            scaling_max_freq: Self::read_policy_property(cpu_id, SCALING_MAX_FREQ)
                .await?
                .parse()?,
        })
    }

    pub(crate) async fn scaling_setspeed(&self, value: u32) -> Result<()> {
        Self::write_policy_property(self.cpu_id, SCALING_SETSPEED, value.to_string()).await
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

    async fn write_policy_property(cpu_id: u32, property: &str, value: String) -> Result<()> {
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
