use anyhow::Result;
use async_std::fs;

const MAXIMUM_FREQUENCY: &'static str = "";
const ONLINE_CPUS: &'static str = "/sys/devices/system/cpu/online";

pub async fn online_cpu_ids(online_cpus: &String) -> Result<Vec<u32>> {
    Ok(online_cpus
        .trim()
        // "1-5,7-9" -> ["1-5", "7-9"]
        .split(",")
        // ["1-5", "7-9", "hello-world", "rust"] -> [["1","5"], ["7","9"], ["hello","world"]]
        .filter_map(|token| token.split_once("-"))
        // [["1","5"], ["7","9"], ["hello","world"]] -> [[1,5], [7,9]]]
        .filter_map(|(first, second)| first.parse::<u32>().ok().zip(second.parse::<u32>().ok()))
        .flat_map(|(first, second)| (first..second).into_iter())
        .collect())
}

pub async fn maximum_frequency() -> Result<u32> {
    Ok(fs::read_to_string(MAXIMUM_FREQUENCY).await?.parse()?)
}

pub async fn online_cpus() -> Result<String, std::io::Error> {
    fs::read_to_string(ONLINE_CPUS).await
}
