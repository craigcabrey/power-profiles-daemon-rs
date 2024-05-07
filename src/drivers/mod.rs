use anyhow::Result;

mod amd;
mod dummy;
mod intel;

pub(crate) struct DriverModule<'a> {
    name: &'a str,
    probe: fn() -> Result<std::sync::Arc<dyn Driver + Sync + Send>>,
}

pub(crate) trait Driver: Send + Sync {
    fn activate(&self, power_profile: crate::types::PowerProfile) -> Result<()>;
    fn current(&self) -> Result<crate::types::InferredPowerProfile>;
    fn name(&self) -> String;
}

pub(crate) fn probe(driver_name: String) -> Result<std::sync::Arc<dyn Driver + Send + Sync>> {
    for driver_module in &[intel::DRIVER, amd::DRIVER, dummy::DRIVER] {
        let candidate = match (driver_module.probe)() {
            Ok(res) => res,
            Err(err) => {
                log::debug!("Skipping driver {}: {:?}", driver_module.name, err);
                continue;
            }
        };

        let name = driver_module.name;
        if (driver_name == "auto".to_string()) || (name == driver_name) {
            log::info!("Using driver {}", name);

            return Ok(candidate);
        }
    }

    Err(anyhow::anyhow!("No driver available".to_string()))
}
