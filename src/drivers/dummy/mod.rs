use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

pub(crate) const DRIVER: super::DriverModule = super::DriverModule {
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

    fn current(&self) -> Result<crate::types::InferredPowerProfile> {
        Ok(crate::types::InferredPowerProfile {
            boost: true,
            energy_preference: crate::types::EnergyPreference::Performance,
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
