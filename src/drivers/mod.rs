use anyhow::Result;
use async_trait::async_trait;

mod cpu;

#[derive(Debug)]
pub(crate) struct DriverModule<'a> {
    name: &'a str,
    probe: fn() -> Result<std::sync::Arc<dyn Driver + Sync + Send>>,
}

#[async_trait]
pub(crate) trait Driver: Send + Sync {
    async fn activate(&self, power_profile: crate::types::PowerProfile) -> Result<()>;
    async fn current(&self) -> Result<crate::types::InferredPowerProfile>;
    fn name(&self) -> String;
}

pub(crate) fn probe() -> Vec<std::sync::Arc<dyn Driver + Send + Sync>> {
    cpu::drivers()
        .into_iter()
        .filter_map(|driver| match (driver.probe)() {
            Ok(res) => {
                log::info!("Using driver");
                Some(res)
            }
            Err(err) => {
                log::debug!("Skipping driver {}: {:?}", driver.name, err);
                None
            }
        })
        .collect()
}
