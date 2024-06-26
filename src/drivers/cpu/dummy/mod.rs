use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use super::types::PowerProfile;

pub struct Driver {}

#[async_trait]
impl crate::drivers::Driver for Driver {
    async fn activate(
        &self,
        _power_profile: &super::super::cpu::types::PowerProfile,
    ) -> Result<()> {
        log::debug!("Activating!");
        Ok(())
    }

    async fn current(&self) -> Result<crate::types::InferredPowerProfile> {
        Ok(crate::types::InferredPowerProfile {
            boost: true,
            energy_preference: super::super::cpu::types::EnergyPreference::Performance,
            maximum_frequency: 4000000,
            scaling_governor: super::super::cpu::types::ScalingGovernor::Performance,
        })
    }

    fn name(&self) -> &str {
        "dummy"
    }
}

pub async fn probe(
    _profiles: &Vec<PowerProfile>,
) -> Result<Arc<dyn crate::drivers::Driver + Send + Sync>> {
    Ok(Arc::new(Driver {}))
}
