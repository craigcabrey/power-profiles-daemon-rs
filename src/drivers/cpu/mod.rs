use anyhow::Result;

use self::types::PowerProfile;

use super::Driver;

mod amd;
pub(crate) mod cpufreq;
mod dummy;
mod intel;
pub(crate) mod types;
mod utils;

pub async fn probe(
    profiles: &Vec<PowerProfile>,
) -> Vec<Result<std::sync::Arc<dyn Driver + Sync + Send>>> {
    vec![
        amd::probe(&profiles).await,
        cpufreq::probe(&profiles).await,
        dummy::probe(&profiles).await,
        intel::probe(&profiles).await,
    ]
}
