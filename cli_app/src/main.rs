use clap::{Arg, Command};
use cli_app::{commands, settings};
use dotenv::dotenv;

fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let command = commands::configure(
        Command::new("Sample CLI application").arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Configuration file location"),
        ),
    );

    let matches = command.get_matches();

    let config_location = matches
        .get_one::<String>("config")
        .map(|s| Some(s.as_str()))
        .unwrap_or(None);
    let settings = settings::Settings::new(config_location, "APP")?;

    commands::handle(&matches, &settings)?;

    Ok(())
}
