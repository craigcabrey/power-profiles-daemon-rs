use core::fmt;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::types::PowerProfile;

pub(crate) mod cpu;

#[async_trait]
pub(crate) trait Driver: Send + Sync {
    async fn activate(&self, power_profile: &PowerProfile) -> Result<()>;
    async fn current(&self) -> Result<crate::types::InferredPowerProfile>;
    fn name(&self) -> &str;
}

#[allow(unused)]
impl fmt::Debug for dyn Driver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Clone)]
pub(crate) struct DriverSet {
    pub cpu: Arc<dyn crate::drivers::Driver + std::marker::Send + Sync>,
}

impl DriverSet {
    pub async fn activate(&self, power_profile: &crate::types::PowerProfile) -> Result<()> {
        self.cpu.activate(power_profile).await
    }
}

pub(crate) async fn probe(settings: &crate::settings::Settings) -> Result<DriverSet> {
    Ok(DriverSet {
        cpu: cpu::probe(
            &settings
                .profiles()
                .clone()
                .into_values()
                .map(|profile| self::cpu::types::PowerProfile::from(profile))
                .collect(),
        )
        .await?,
    })
}
