use clap::{App, Arg, ArgGroup, SubCommand};

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
        .subcommand(channel::channel_subcommand());

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
    }

    fn list() -> App<'static, 'static> {
        SubCommand::with_name("list")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about("Channel api")
            .long_about("This command manipulate channel api.")
            .visible_alias("ls")
    }
}
