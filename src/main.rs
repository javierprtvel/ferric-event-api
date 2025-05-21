use dotenv::dotenv;
use log::error;

use ferric_event_api::init_tracing;
use ferric_event_api::load_config;
use ferric_event_api::serve_app;

fn main() -> anyhow::Result<()> {
    dotenv().ok();

    init_tracing()
        .and_then(|_| load_config())
        .and_then(serve_app)
        .inspect_err(|e| error!("Error while starting application: {e:?}\n"))?;

    Ok(())
}
