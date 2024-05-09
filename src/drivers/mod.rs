use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use self::cpu::types::PowerProfile;

pub(crate) mod cpu;

#[async_trait]
pub(crate) trait Driver: Send + Sync {
    async fn activate(&self, power_profile: cpu::types::PowerProfile) -> Result<()>;
    async fn current(&self) -> Result<crate::types::InferredPowerProfile>;
    fn name(&self) -> String;
}

#[derive(Clone)]
pub(crate) struct DriverSet {
    pub cpu: Arc<dyn crate::drivers::Driver + std::marker::Send + Sync>,
}

impl DriverSet {
    pub async fn activate(&self, power_profile: crate::types::PowerProfile) -> Result<()> {
        self.cpu.activate(power_profile.cpu).await

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

pub(crate) async fn probe(settings: &crate::settings::Settings) -> Result<DriverSet> {
    let cpu_drivers = cpu::probe(
        &settings
            .profiles()
            .clone()
            .into_values()
            .map(|profile| PowerProfile::from(profile))
            .collect(),
    )
    .await
    .into_iter()
    .filter_map(|driver| match driver {
        Ok(res) => {
            log::trace!("Loaded driver {:#?}", res.name());
            Some(res)
        }
        Err(err) => {
            log::debug!("Skipping driver: {}", err);
            None
        }
    })
    .collect::<Vec<_>>();

    // FIXME
    let cpu_driver = cpu_drivers.into_iter().next();

    Ok(DriverSet {
        cpu: cpu_driver.unwrap(),
    })
}
