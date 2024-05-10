use std::{collections::HashMap, sync::Arc};

use anyhow::Result;

use super::types::PowerProfile;

mod pstate;

const SCALING_DRIVER_PATH: &str = "/sys/devices/system/cpu/cpufreq/policy0/scaling_driver";

pub(crate) async fn probe(
    profiles: &Vec<PowerProfile>,
) -> Result<Arc<dyn crate::drivers::Driver + Send + Sync>> {
    let profile_driver_settings: HashMap<String, pstate::DriverSettings> = profiles
        .into_iter()
        .filter_map(|profile| match &profile.driver_options {
            Some(options) => match <config::Value as Clone>::clone(&options)
                .try_deserialize::<pstate::DriverSettings>()
            {
                Ok(res) => Some((profile.name.clone().unwrap(), res)),
                Err(err) => {
                    log::warn!("Failed to apply driver options: {:#?}", err);
                    None
                }
            },
            None => None,
        })
        .collect();

    let driver = std::fs::read_to_string(SCALING_DRIVER_PATH)?;

    match driver.trim() {
        "amd-pstate" => Ok(Arc::new(
            pstate::Driver::new(false, "amd-pstate".to_string(), profile_driver_settings).await?,
        )),
        "amd-pstate-epp" => Ok(Arc::new(
            pstate::Driver::new(false, "amd-pstate-epp".to_string(), profile_driver_settings)
                .await?,
        )),
        _ => Err(anyhow::anyhow!("unsupported driver {}", driver.trim())),
    }
}
