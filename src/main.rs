use std::io::IsTerminal;

use tracing_subscriber::EnvFilter;
use clap::{Parser, CommandFactory};

mod build;
// mod list;
// mod query_files;

#[derive(clap::Parser)]
#[command(version, about)]
struct Args {
  #[arg(short='F', long)]
  /// ignored
  files: bool,

  #[arg(short, long)]
  /// List the files owned by the queried package.
  list: bool,

  #[arg(short='x', long)]
  /// Interpret each query as a Rust regular expression.
  regex: bool,

  #[arg(short='y', long, action = clap::ArgAction::Count)]
  /// Refresh & rebuild databases; give twice to force
  refresh: u8,

  #[arg(value_name="QUERY")]
  /// the query
  queries: Vec<String>,
}

fn main() -> eyre::Result<()> {
  let filter = EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::from("warn"));
  let isatty = std::io::stderr().is_terminal();
  let fmt = tracing_subscriber::fmt::fmt()
    .with_writer(std::io::stderr)
    .with_env_filter(filter)
    .with_ansi(isatty);
  if isatty {
    fmt.with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
        String::from("%Y-%m-%d %H:%M:%S%.6f %z")))
      .init();
  } else {
    fmt.without_time().init();
  }

  let args = Args::parse();

  if args.refresh > 2 {
    Args::command()
      .error(clap::error::ErrorKind::InvalidValue, "refresh can give twice at most")
      .exit();
  }

  if args.refresh > 0 {
    build::refresh(args.refresh == 2)?;
  }

  Ok(())
}

