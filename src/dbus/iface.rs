//! # D-Bus interface proxy for: `org.freedesktop.UPower.PowerProfiles`
//!
//! This code was generated by `zbus-xmlgen` `4.1.0` from D-Bus introspection data.
//! Source: `Interface '/org/freedesktop/UPower/PowerProfiles' from service 'org.freedesktop.UPower.PowerProfiles' on system bus`.
//!
//! You may prefer to adapt it, instead of using it verbatim.
//!
//! More information can be found in the [Writing a client proxy] section of the zbus
//! documentation.
//!
//! This type implements the [D-Bus standard interfaces], (`org.freedesktop.DBus.*`) for which the
//! following zbus API can be used:
//!
//! * [`zbus::fdo::PropertiesProxy`]
//! * [`zbus::fdo::IntrospectableProxy`]
//! * [`zbus::fdo::PeerProxy`]
//!
//! Consequently `zbus-xmlgen` did not generate code for the above interfaces.
//!
//! [Writing a client proxy]: https://dbus2.github.io/zbus/client.html
//! [D-Bus standard interfaces]: https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces,
use zbus::proxy;

pub(crate) trait PowerProfiles {
    /// HoldProfile method
    fn hold_profile(&self, profile: &str, reason: &str, application_id: &str) -> zbus::Result<u32>;

    /// ReleaseProfile method
    fn release_profile(&self, cookie: u32) -> zbus::Result<()>;

    /// ProfileReleased signal
    #[zbus(signal)]
    fn profile_released(&self) -> zbus::Result<()>;

    /// Actions property
    #[zbus(property)]
    fn actions(&self) -> zbus::Result<Vec<String>>;

    /// ActiveProfile property
    #[zbus(property)]
    fn active_profile(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn set_active_profile(&self, value: &str) -> zbus::Result<()>;

    /// ActiveProfileHolds property
    #[zbus(property)]
    fn active_profile_holds(&self) -> zbus::Result<Vec<std::collections::HashMap<String, String>>>;

    /// PerformanceDegraded property
    #[zbus(property)]
    fn performance_degraded(&self) -> zbus::Result<String>;

    /// PerformanceInhibited property
    #[zbus(property)]
    fn performance_inhibited(&self) -> zbus::Result<String>;

    /// Profiles property
    #[zbus(property)]
    fn profiles(&self) -> zbus::Result<Vec<std::collections::HashMap<String, String>>>;

    /// Version property
    #[zbus(property)]
    fn version(&self) -> zbus::Result<String>;
}
