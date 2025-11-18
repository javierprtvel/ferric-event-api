use crate::{settings::Settings, state::ApplicationState};

use clap::{Arg, ArgMatches, Command, value_parser};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tower_http::trace::TraceLayer;
use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub const COMMAND_NAME: &str = "serve";

pub fn configure() -> Command {
    Command::new(COMMAND_NAME).about("Start HTTP server").arg(
        Arg::new("port")
            .short('p')
            .long("port")
            .value_name("PORT")
            .help("TCP port to listen on")
            .default_value("8080")
            .value_parser(value_parser!(u16)),
    )
}

pub fn handle(matches: &ArgMatches, settings: &Settings) -> anyhow::Result<()> {
    let port: u16 = *matches.get_one("port").unwrap_or(&8080);

    start_tokio(port, settings)?;

    Ok(())
}

fn start_tokio(port: u16, settings: &Settings) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let subscriber = tracing_subscriber::registry()
                .with(LevelFilter::from_level(Level::TRACE))
                .with(fmt::Layer::default());

            subscriber.init();

            let db_url = settings
                .database
                .url
                .clone()
                .expect("Database URL is not set");
            let pool = sqlx::MySqlPool::connect(&db_url).await?;

            let state = Arc::new(ApplicationState::new(settings, pool)?);
            let router = crate::api::configure(state.clone()).layer(TraceLayer::new_for_http());

            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, router.into_make_service()).await?;

            Ok::<(), anyhow::Error>(())
        })?;

    Ok(())
}
