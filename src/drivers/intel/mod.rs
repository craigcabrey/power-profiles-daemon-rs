use std::sync::Arc;

use anyhow::Result;

mod pstate;

pub(crate) const DRIVER: super::DriverModule = super::DriverModule {
    name: "intel-pstate",
    probe: probe,
};
const SCALING_DRIVER_PATH: &str = "/sys/devices/system/cpu/cpufreq/policy0/scaling_driver";

pub fn probe() -> Result<Arc<dyn crate::drivers::Driver + Send + Sync>> {
    let driver = std::fs::read_to_string(SCALING_DRIVER_PATH)?;

    match driver.trim() {
        "intel_pstate" => Ok(Arc::new(pstate::Driver::new(
            false,
            DRIVER.name.to_string(),
        )?)),
        _ => Err(anyhow::anyhow!("unsupported driver {}", driver.trim())),
    }
}
