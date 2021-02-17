use clap::Clap;
use std::path::PathBuf;

#[derive(Clap)]
struct Opts {
    // This is passed in by cargo.
    _my_command: String,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Clap)]
enum Commands {
    Build(BuildOpts),
}

#[derive(Clap)]
struct BuildOpts {
    #[clap(long)]
    manifest_path: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    match opts.command {
        Commands::Build(c) => {
            cargo_megadrive::Builder::new(c.manifest_path)?
                .build()?;
        },
    }

    Ok(())
}