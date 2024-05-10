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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("{} version {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let args = Args::parse();
    let settings = settings::Settings::build(&args.config)?;
    let driver_set = drivers::probe(&settings).await?;

    log::trace!("Loaded {:#?}", settings);

    let handler = dbus::Handler::new(driver_set.clone(), settings.clone());
    let legacy_handler = dbus::legacy::Handler::new(driver_set, settings);

    let mut bus_type = connection::Builder::system
        as fn() -> Result<zbus::ConnectionBuilder<'static>, zbus::Error>;

    if args.user {
        log::info!("Running on the user session bus, use for development only");

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
