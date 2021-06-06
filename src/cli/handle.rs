use super::channel;
use anyhow::Result;
use clap::ArgMatches;
use rust_traq::apis::configuration::Configuration;

pub async fn handle_matches(
    conf: &Configuration,
    matches: &ArgMatches<'_>,
    cmd: &str,
) -> Result<()> {
    match cmd {
        "channel" => {
            if let Some(cmd) = matches.subcommand_name() {
                let m = matches.subcommand_matches(cmd).unwrap();
                channel::channel(conf, m, cmd).await
            } else {
                channel::channel(conf, matches, "list").await
            }
        }
        x => {
            dbg!("{}", x);
            Ok(())
        }
    }
}
