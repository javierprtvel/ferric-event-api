mod application;
mod domain;
mod infrastructure;

use dotenv::dotenv;
use log::error;

use infrastructure::init_tracing;
use infrastructure::load_config;
use infrastructure::serve_app;

fn main() -> anyhow::Result<()> {
    dotenv().ok();

    init_tracing()
        .and_then(|_| load_config())
        .and_then(serve_app)
        .inspect_err(|e| error!("Error while starting application: {e:?}\n"))?;

    Ok(())
}
