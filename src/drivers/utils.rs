use anyhow::Result;
use async_std::fs;
use futures::{StreamExt, TryStreamExt};

use crate::types::EnergyPreference;

const MAXIMUM_FREQUENCY: &'static str = "/sys/devices/system/cpu/cpufreq/policy0/scaling_max_freq";
const ONLINE_CPUS: &'static str = "/sys/devices/system/cpu/online";

pub(crate) async fn activate_energy_preference(energy_preference: EnergyPreference) -> Result<()> {
    log::debug!("Writing to /sys/devices/system/cpu/cpufreq/policy*/energy_performance_preference");

    futures::stream::iter(online_cpu_id_iter(&online_cpus().await?)?)
        .then(|core_id| async move {
            fs::write(
                format!(
                    "/sys/devices/system/cpu/cpufreq/policy{:?}/energy_performance_preference",
                    core_id,
                ),
                energy_preference.to_string(),
            )
            .await
        })
        .try_collect()
        .await
        .map_err(anyhow::Error::from)
}

pub(crate) async fn activate_scaling_governor(
    scaling_governor: crate::types::ScalingGovernor,
) -> Result<()> {
    log::debug!("Writing to /sys/devices/system/cpu/cpufreq/policy*/scaling_governor");

    futures::stream::iter(online_cpu_id_iter(&online_cpus().await?)?)
        .then(|core_id| async move {
            fs::write(
                format!(
                    "/sys/devices/system/cpu/cpufreq/policy{}/scaling_governor",
                    core_id,
                ),
                scaling_governor.to_string(),
            )
            .await
        })
        .try_collect()
        .await
        .map_err(anyhow::Error::from)
}

pub(crate) async fn maximum_frequency() -> Result<u32> {
    Ok(fs::read_to_string(MAXIMUM_FREQUENCY)
        .await?
        .trim()
        .parse()?)
}

async fn online_cpus() -> Result<String, std::io::Error> {
    fs::read_to_string(ONLINE_CPUS).await
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
        .flat_map(|(first, second)| (first..second).into_iter()))
}
