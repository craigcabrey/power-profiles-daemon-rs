use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::drivers;

pub(crate) const DRIVER: drivers::DriverModule = drivers::DriverModule {
    name: "dummy",
    probe: probe,
};
pub struct Driver {}

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

pub fn probe() -> Result<Arc<dyn crate::drivers::Driver + Send + Sync>> {
    Ok(Arc::new(Driver {}))
}
