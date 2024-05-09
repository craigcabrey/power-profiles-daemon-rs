use std::future::pending;

use anyhow::Result;
use clap::Parser;
use zbus::connection;

mod dbus;
mod drivers;
mod settings;
mod types;

/// Drop in replacement for power-profiles-daemon
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to config file
    #[arg(short, long, default_value = "config.json")]
    config: String,

    /// Name of the driver to use
    #[arg(long, default_value = "auto")]
    driver: String,

    /// Best effort to avoid mutable operations
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// Disable upower interface handler
    #[arg(long, default_value_t = false)]
    disable_upower: bool,

    /// Disable legacy interface handler
    #[arg(long, default_value_t = false)]
    disable_legacy: bool,

    /// Launch on the user session bus (useful for development)
    #[arg(long, default_value_t = false)]
    user: bool,
}

#[async_std::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let args = Args::parse();
    let settings = settings::Settings::build(&args.config)?;

    log::trace!("Loaded {:#?}", settings);

    let handler = dbus::Handler::new(drivers::probe(args.driver.clone())?, settings.clone());
    let legacy_handler = dbus::legacy::Handler::new(drivers::probe(args.driver.clone())?, settings);

    let mut bus_type = connection::Builder::system
        as fn() -> Result<zbus::ConnectionBuilder<'static>, zbus::Error>;

    if args.user {
        log::info!("Using the user session bus");

        bus_type = connection::Builder::session
            as fn() -> Result<zbus::ConnectionBuilder<'static>, zbus::Error>;
    }

    // Hold references to all DBus connections, otherwise they die
    let mut connections = Vec::new();

    if !args.disable_upower {
        log::info!("Starting upower interface handler");

        connections.push(
            bus_type()?
                .name("org.freedesktop.UPower.PowerProfiles")?
                .serve_at("/org/freedesktop/UPower/PowerProfiles", handler)?
                .build()
                .await?,
        );
    }

    if !args.disable_legacy {
        log::info!("Starting legacy interface handler");

        connections.push(
            bus_type()?
                .name("net.hadess.PowerProfiles")?
                .serve_at("/net/hadess/PowerProfiles", legacy_handler)?
                .build()
                .await?,
        );
    }

    Ok(pending::<()>().await)
}
