use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::drivers::cpu;

pub struct Driver {}

#[async_trait]
impl crate::drivers::Driver for Driver {
    async fn activate(&self, _power_profile: &crate::types::PowerProfile) -> Result<()> {
        log::debug!("Activating!");
        Ok(())
    }

    async fn current(&self) -> Result<crate::types::InferredPowerProfile> {
        Ok(crate::types::InferredPowerProfile {
            cpu: Some(cpu::types::InferredPowerProfile { cpu_profiles: None }),
        })
    }

    fn name(&self) -> &str {
        "dummy"
    }
}

pub async fn probe(
    _profiles: &Vec<cpu::types::PowerProfile>,
) -> Result<Arc<dyn crate::drivers::Driver + Send + Sync>> {
    Ok(Arc::new(Driver {}))
}
