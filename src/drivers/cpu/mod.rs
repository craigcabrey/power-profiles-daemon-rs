use anyhow::Result;

use super::Driver;

mod amd;
pub(crate) mod cpufreq;
mod dummy;
mod intel;
pub(crate) mod types;
mod utils;

pub async fn probe() -> Vec<Result<std::sync::Arc<dyn Driver + Sync + Send>>> {
    vec![
        amd::probe().await,
        cpufreq::probe().await,
        dummy::probe().await,
        intel::probe().await,
    ]
}
