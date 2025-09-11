use clap::Parser;

#[derive(clap::Parser)]
struct CommandParser {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Print working directory
    Pwd,
    /// Change working directory
    Cd {
        /// Destination directory.
        /// If not specified, go to $HOME.
        /// If "-" is specified, goto $OLDPWD (previous dir).
        dir: Option<String>,
    },
    /// List files
    Ls { path: Vec<String> },
}

pub fn exec(cmdline: &str) -> anyhow::Result<()> {
    let cd = std::env::current_dir()?;
    println!("{}$ {cmdline}", cd.to_string_lossy());

    let arg0 = std::iter::once("CMDLINE");
    let tokens = cmdline.split_whitespace();
    let parsed = CommandParser::try_parse_from(arg0.chain(tokens))?;

    match parsed.command {
        Commands::Pwd => cmd_pwd(),
        Commands::Cd { dir: _ } => todo!(),
        Commands::Ls { path: _ } => todo!(),
    }
}

fn cmd_pwd() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    println!("{}", dir.as_os_str().to_string_lossy());

    Ok(())
}
