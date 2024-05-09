use anyhow::Result;
use async_std::fs;
use futures::{StreamExt, TryStreamExt};

// If no max frequency specified, reset it to max by specifying a ludicrous value, like 100 GHz
const MAX_RESET_FREQUENCY: u32 = 100000000;
const MAXIMUM_FREQUENCY: &'static str = "/sys/devices/system/cpu/cpufreq/policy0/scaling_max_freq";
const ONLINE_CPUS: &'static str = "/sys/devices/system/cpu/online";

pub(crate) async fn activate_energy_preference(
    energy_preference: super::types::EnergyPreference,
) -> Result<()> {
    log::info!("Activating energy preference {:?}", energy_preference);

    futures::stream::iter(online_cpu_id_iter(&online_cpus().await?)?)
        .then(|core_id| async move {
            log::debug!(
                "Writing {} to /sys/devices/system/cpu/cpufreq/policy{}/energy_performance_preference",
                energy_preference.to_string(),
                core_id,
            );

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

pub(crate) async fn activate_maximum_frequency(maximum_frequency: Option<u32>) -> Result<()> {
    let value = match maximum_frequency {
        Some(value) => {
            log::info!("Activating maximum frequency {}", value);
            value
        }
        None => {
            log::info!("Resetting maximum frequency");
            MAX_RESET_FREQUENCY
        }
    };

    futures::stream::iter(online_cpu_id_iter(&online_cpus().await?)?)
        .then(|core_id| async move {
            log::debug!(
                "Writing {} to /sys/devices/system/cpu/cpufreq/policy{}/scaling_max_freq",
                value,
                core_id,
            );

            fs::write(
                format!(
                    "/sys/devices/system/cpu/cpufreq/policy{}/scaling_max_freq",
                    core_id,
                ),
                value.to_string(),
            )
            .await
        })
        .try_collect()
        .await
        .map_err(anyhow::Error::from)
}

pub(crate) async fn activate_scaling_governor(
    scaling_governor: super::types::ScalingGovernor,
) -> Result<()> {
    log::info!("Activating scaling governor {:?}", scaling_governor);

    futures::stream::iter(online_cpu_id_iter(&online_cpus().await?)?)
        .then(|core_id| async move {
            log::debug!(
                "Writing {} to /sys/devices/system/cpu/cpufreq/policy{}/scaling_governor",
                scaling_governor.to_string(),
                core_id,
            );

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

pub(crate) async fn online_cpus() -> Result<String, std::io::Error> {
    fs::read_to_string(ONLINE_CPUS).await
}

pub(crate) fn online_cpu_id_iter(online_cpus: &String) -> Result<impl Iterator<Item = u32> + '_> {
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
