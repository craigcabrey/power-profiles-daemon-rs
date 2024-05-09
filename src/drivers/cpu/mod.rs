mod amd;
pub(crate) mod cpufreq;
mod dummy;
mod intel;
mod types;
mod utils;

pub fn drivers() -> Vec<super::DriverModule<'static>> {
    vec![amd::DRIVER, cpufreq::DRIVER, intel::DRIVER, dummy::DRIVER]
}
