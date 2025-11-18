mod application;
mod domain;
mod infrastructure;

use infrastructure::serve_app;

fn main() -> anyhow::Result<()> {
    serve_app()?;
    Ok(())
}
