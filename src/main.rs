mod application;
mod domain;
mod infrastructure;

use dotenv::dotenv;
use infrastructure::load_config;
use infrastructure::serve_app;

const APP_ENV_PREFIX: &'static str = "APP";

fn main() -> anyhow::Result<()> {
    dotenv()?;
    let app_config = load_config(APP_ENV_PREFIX)?;
    serve_app(app_config)?;
    Ok(())
}
