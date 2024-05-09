use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

mod cpu;

#[async_trait]
pub(crate) trait Driver: Send + Sync {
    async fn activate(&self, power_profile: crate::types::PowerProfile) -> Result<()>;
    async fn current(&self) -> Result<crate::types::InferredPowerProfile>;
    fn name(&self) -> String;
}

#[derive(Clone)]
pub(crate) struct DriverSet {
    pub cpu: Arc<dyn crate::drivers::Driver + std::marker::Send + Sync>,
}

impl DriverSet {
    pub async fn activate(&self, power_profile: crate::types::PowerProfile) -> Result<()> {
        self.cpu.activate(power_profile).await

        // futures::future::join_all(
        //     self.cpu
        //         .clone()
        //         .into_iter()
        //         .map(|driver| driver.activate(power_profile)),
        // )
        // .await
        // .into_iter()
        // .collect::<Vec<Result<_, _>>>();
    }
}

pub(crate) async fn probe() -> Result<DriverSet> {
    let cpu_drivers = cpu::probe()
        .await
        .into_iter()
        .filter_map(|driver| match driver {
            Ok(res) => {
                log::info!("Using driver {}", res.name());
                Some(res)
            }
            Err(err) => {
                log::debug!("Skipping driver: {}", err);
                None
            }
        })
        .collect::<Vec<_>>();

    let cpu_driver = cpu_drivers.into_iter().next();

    Ok(DriverSet {
        cpu: cpu_driver.unwrap(),
    })
}
