use std::collections::HashMap;

use anyhow::Result;
use config::Config;
use serde::Deserialize;
use serde_with::{serde_as, KeyValueMap};

use crate::types::{InferredPowerProfile, PowerProfile};

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Settings {
    pub(crate) default: String,
    profiles: HashMap<String, PowerProfile>,
}

impl Settings {
    fn new(default: String, profiles: HashMap<String, PowerProfile>) -> Result<Self> {
        let instance = Self {
            default: default,
            profiles: profiles,
        };

        match instance.profiles.get(&instance.default) {
            Some(_) => Ok(instance),
            None => Err(anyhow::anyhow!(
                "Default profile {} is not configured!",
                instance.default
            )),
        }
    }
}

#[serde_as]
#[derive(Clone, Debug, Deserialize)]
struct RawSettings {
    default: String,
    #[serde_as(as = "KeyValueMap<_>")]
    profiles: Vec<PowerProfile>,
}

impl TryInto<Settings> for RawSettings {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Settings> {
        Settings::new(
            self.default,
            self.profiles
                .into_iter()
                .map(|profile| (profile.name.clone(), profile.into()))
                .collect(),
        )
    }
}

impl Settings {
    pub fn build(config_path: &str) -> Result<Self> {
        Config::builder()
            .add_source(config::File::with_name(config_path))
            .add_source(config::Environment::with_prefix("PPD"))
            .build()?
            .try_deserialize::<RawSettings>()?
            .try_into()
    }

    pub fn profiles(&self) -> &HashMap<String, PowerProfile> {
        &self.profiles
    }

    pub fn profile_by_name(&self, profile_name: &String) -> Option<&PowerProfile> {
        self.profiles.get(profile_name)
    }

    pub fn profile_by_inferred(
        &self,
        inferred_profile: InferredPowerProfile,
    ) -> Option<PowerProfile> {
        for profile in self.profiles.values().into_iter() {
            if *profile == inferred_profile {
                return Some(profile.clone());
            }
        }

        None
    }
}
