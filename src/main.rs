// Copyright (C) 2025 lilydjwg
// 
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

use std::io::IsTerminal;

use tracing_subscriber::EnvFilter;
use clap::{Parser, CommandFactory};

mod build;
mod list;
mod files;
mod query_files;
mod installed;

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
  /// Interpret each query as a POSIX extended regular expression.
  regex: bool,

  #[arg(short, long)]
  /// Do not output colors and file paths
  quiet: bool,

  #[arg(short='y', long, action = clap::ArgAction::Count)]
  /// Refresh & rebuild databases; give twice to force
  refresh: u8,

  #[arg(long, action = clap::ArgAction::Count)]
  /// rebuild databases only without refreshing; give twice to force
  update_db: u8,

  #[arg(value_name="QUERY")]
  /// The query; unlike pacman, globs (*?[]) are supported in non-regex mode
  query: Vec<String>,
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
  if args.update_db > 2 {
    Args::command()
      .error(clap::error::ErrorKind::InvalidValue, "update-db can give twice at most")
      .exit();
  }

  if args.refresh > 0 {
    build::refresh(args.refresh == 2)?;
  } else if args.update_db > 0 {
    build::update_db(args.update_db == 2)?;
  } else if args.list {
    list::list_packages(&args.query, args.quiet)?;
  } else {
    query_files::query_files(&args.query, args.regex, args.quiet)?;
  }

  Ok(())
}

