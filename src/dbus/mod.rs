use std::collections::HashMap;

use zbus::interface;

use crate::{drivers, settings::Settings, types::PowerProfileHold};

#[derive(Clone)]
pub(crate) struct Handler {
    driver: std::sync::Arc<dyn drivers::Driver + Send + Sync>,
    profile_holds: HashMap<u32, PowerProfileHold>,
    settings: Settings,
}

impl Handler {
    pub fn new(driver: std::sync::Arc<dyn drivers::Driver + Send + Sync>, settings: Settings) -> Self {
        Self{
            driver: driver,
            profile_holds: HashMap::new(),
            settings,
        }
    }
}

#[interface(name = "org.freedesktop.UPower.PowerProfiles")]
impl Handler {
    #[zbus(property)]
    async fn actions(&self) -> Vec<String> {
        vec![]
    }

    #[zbus(property)]
    async fn active_profile(&self) -> anyhow::Result<String, zbus::fdo::Error> {
        match self.driver.current() {
            Ok(profile) => {
                match self.settings.profile_by_inferred(profile) {
                    Some(profile) => Ok(profile.name),
                    None => {
                        log::warn!("Unable to determine current profile");
                        Ok(self.settings.default.clone())
                    }
                }
            },
            Err(err) => Err(zbus::fdo::Error::Failed(format!("{:?}", err)))?,
        }
    }

    #[zbus(property)]
    async fn set_active_profile(&mut self, name: String) -> anyhow::Result<(), zbus::fdo::Error> {
        log::info!("Activating profile: {}", name);

        match self.settings.profile_by_name(&name) {
            Some(profile) => {
                match self.driver.activate(profile) {
                    Ok(()) => Ok(()),
                    Err(err) => Err(zbus::fdo::Error::Failed(format!("{:?}", err))),
                }
            },
            None => {
                log::warn!("Received request to activate missing profile {}", name);

                Err(
                    zbus::fdo::Error::InvalidArgs(format!("No such profile"))
                )
            },
        }
    }

    #[zbus(property)]
    async fn active_profile_holds(&self) -> Vec<HashMap<String, String>> {
        self.profile_holds.clone().into_values().map(|profile_hold| {
            HashMap::from([
                ("ApplicationId".to_string(), profile_hold.application_id),
                ("Profile".to_string(), profile_hold.profile),
                ("Reason".to_string(), profile_hold.reason),
            ])
        }).collect()
    }

    #[zbus(property)]
    async fn performance_degraded(&self) -> &str {
        // - "lap-detected" (the computer is sitting on the user's lap)
        // - "high-operating-temperature" (the computer is close to overheating)
        // - "" (the empty string, if not performance is not degraded)

        ""
    }

    #[zbus(property)]
    async fn performance_inhibited(&self) -> &str {
        // deprecated property
        ""
    }

    #[zbus(property)]
    async fn profiles(&self) -> Vec<HashMap<String, String>> {
        self.settings.profiles().clone().into_values().map(|profile| {
            HashMap::from([
                ("Profile".to_string(), profile.name),
                ("CpuDriver".to_string(), self.driver.name()),
                ("PlatformDriver".to_string(), "placeholder".to_string()),
                ("Driver".to_string(), "multiple".to_string()),
            ])
        }).collect()
    }

    #[zbus(property)]
    async fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    #[zbus(signal)]
    async fn profile_released(ctxt: &zbus::SignalContext<'_>) -> zbus::Result<()>;

    fn hold_profile(&mut self, profile: &str, reason: &str, application_id: &str) -> anyhow::Result<u32, zbus::fdo::Error> {
        let cookie = 0;

        self.profile_holds.insert(
            cookie,    
            PowerProfileHold::new(
                application_id.to_owned(),
                profile.to_owned(),
                reason.to_owned(),
            ),
        );

        match self.settings.profiles().get(profile) {
            Some(profile) => {
                match self.driver.activate(profile.clone()) {
                    Ok(()) => Ok(cookie),
                    Err(err) => Err(zbus::fdo::Error::Failed(err.to_string())),
                }
            },
            _ => Err(zbus::fdo::Error::InvalidArgs("No such profile".to_string())),
        }
    }

    fn release_profile(&mut self, cookie: u32) {
        self.profile_holds.remove(&cookie);
    }
}