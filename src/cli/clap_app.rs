use clap::{App, Arg, SubCommand};

const BANNER: &str = "
            _ 
 ___ ___ __(_)
/ _ `/ // / / 
\\_, /\\_,_/_/  
 /_/          
";

pub fn clap_app() -> App<'static, 'static> {
    let clap_app = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .usage("Press `?` while running the app to see keybindings")
        .before_help(BANNER)
        .after_help("after help")
        .subcommand(channel::channel_subcommand())
        .subcommand(notify::notify_subcommand());

    clap_app
}

mod channel {
    use super::*;

    pub fn channel_subcommand() -> App<'static, 'static> {
        SubCommand::with_name("channel")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about("Channel api")
            .long_about("This command manipulate channel api.")
            .visible_alias("ch")
            .subcommand(list())
            .subcommand(cd())
    }

    fn list() -> App<'static, 'static> {
        SubCommand::with_name("list")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about("Channel api")
            .long_about("This command manipulate channel api.")
            .visible_alias("ls")
            .arg(
                Arg::with_name("recursive")
                    .short("r")
                    .long("recursive")
                    .help("show channel recursively"),
            )
            .arg(
                Arg::with_name("full")
                    .short("f")
                    .long("full-path")
                    .help("show full path of channel"),
            )
            .arg(Arg::with_name("channel_name").help("specify channel name"))
    }

    fn cd() -> App<'static, 'static> {
        SubCommand::with_name("cd")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about("change channel")
            .long_about("change current channel")
            .arg(
                Arg::with_name("channel_name")
                    .help("channel name")
                    .required(true)
                    .multiple(false),
            )
    }
}

mod notify {
    use super::*;

    pub fn notify_subcommand() -> App<'static, 'static> {
        SubCommand::with_name("notify")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about("notification api")
            .long_about("This command manipulate notification api.")
            .visible_alias("notif")
            .arg(
                Arg::with_name("level")
                    // .short("l")
                    // .long("level")
                    .help("notification level. 0: not subscribe, 1: unread only, 2: on")
                    // .takes_value(true)
                    .multiple(false)
                    .required(true),
            )
            .arg(
                Arg::with_name("channel_names")
                    // .long("channel-names")
                    .help("specify channel names")
                    .takes_value(true)
                    .multiple(true), // .required(true),
            )
    }
}
