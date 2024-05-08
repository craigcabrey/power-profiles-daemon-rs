use anyhow::Result;
use async_std::fs;

const ONLINE_CPUS: &'static str = "/sys/devices/system/cpu/online";

pub async fn online_cpus() -> Result<String, std::io::Error> {
    fs::read_to_string(ONLINE_CPUS).await
}

pub async fn cores(online_cpus: &String) -> Result<Vec<i32>> {
    Ok(online_cpus
        .trim()
        // "1-5,7-9" -> ["1-5", "7-9"]
        .split(",")
        // ["1-5", "7-9", "hello-world", "rust"] -> [["1","5"], ["7","9"], ["hello","world"]]
        .filter_map(|token| token.split_once("-"))
        // [["1","5"], ["7","9"], ["hello","world"]] -> [[1,5], [7,9]]]
        .filter_map(|(first, second)| first.parse::<i32>().ok().zip(second.parse::<i32>().ok()))
        .flat_map(|(first, second)| (first..second).into_iter())
        .collect())
}
