use std::sync::Arc;

use anyhow::Result;

mod pstate;

const SCALING_DRIVER_PATH: &str = "/sys/devices/system/cpu/cpufreq/policy0/scaling_driver";

pub async fn probe() -> Result<Arc<dyn crate::drivers::Driver + Send + Sync>> {
    let driver = async_std::fs::read_to_string(SCALING_DRIVER_PATH).await?;

    match driver.trim() {
        "intel_pstate" => Ok(Arc::new(pstate::Driver::new(false).await?)),
        _ => Err(anyhow::anyhow!("unsupported driver {}", driver.trim())),
    }
}
