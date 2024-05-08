use std::sync::Arc;

use anyhow::Result;

mod pstate;

const SCALING_DRIVER_PATH: &str = "/sys/devices/system/cpu/cpufreq/policy0/scaling_driver";
pub(crate) const DRIVER: super::DriverModule = super::DriverModule {
    name: "amd-pstate",
    probe: probe,
};

// TODO: asyncify driver probing
pub fn probe() -> Result<Arc<dyn crate::drivers::Driver + Send + Sync>> {
    futures::executor::block_on(aprobe())
}

async fn aprobe() -> Result<Arc<dyn crate::drivers::Driver + Send + Sync>> {
    let driver = std::fs::read_to_string(SCALING_DRIVER_PATH)?;

    match driver.trim() {
        "amd-pstate" => Ok(Arc::new(
            pstate::Driver::new(false, "amd-pstate".to_string()).await?,
        )),
        "amd-pstate-epp" => Ok(Arc::new(
            pstate::Driver::new(false, "amd-pstate-epp".to_string()).await?,
        )),
        _ => Err(anyhow::anyhow!("unsupported driver {}", driver.trim())),
    }
}
