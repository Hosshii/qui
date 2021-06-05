use clap::ArgMatches;

pub fn handle_matches(matches: &ArgMatches<'_>, cmd: &str) {
    match cmd {
        "channel" => {
            if let Some(cmd) = matches.subcommand_name() {
                let m = matches.subcommand_matches(cmd).unwrap();
                channel::channel(m, cmd);
            } else {
                channel::channel(matches, "list")
            }
        }
        x => {
            dbg!("{}", x);
        }
    }
}

mod channel {
    use super::*;

    pub fn channel(matches: &ArgMatches<'_>, cmd: &str) {
        match cmd {
            "list" => {
                dbg!("list!!");
            }
            x => {
                dbg!("{}", x);
            }
        }
    }

    fn get_channels() {}
}
