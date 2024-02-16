use std::io::IsTerminal;
use std::ffi::OsStr;

use tracing_subscriber::EnvFilter;
use clap::Parser;

mod files;
mod list;
mod query_files;

#[derive(clap::Parser)]
struct Args {
  #[arg(short, long)]
  /// List the files owned by the queried package.
  list: bool,

  #[arg(short='x', long)]
  /// Interpret each query as a Rust regular expression.
  regex: bool,

  #[arg(value_name="QUERY")]
  /// the query
  queries: Vec<String>,
}

type Matcher = dyn Fn(&str) -> bool;

fn main() -> eyre::Result<()> {
  let filter = EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::from("warn"));
  let isatty = std::io::stderr().is_terminal();
  let fmt = tracing_subscriber::fmt::fmt()
    .with_writer(std::io::stderr)
    .with_env_filter(filter)
    .with_ansi(isatty);
  if isatty {
    fmt.init();
  } else {
    fmt.without_time().init();
  }

  let args = Args::parse();

  let matcher = if args.regex {
    let regex = regex::RegexSet::new(&args.queries)?;
    Box::new(move |file: &str| regex.is_match(file)) as Box<Matcher>
  } else {
    let queries = args.queries.clone();
    Box::new(move |file: &str| {
      queries.iter().any(|pat|
        if file.ends_with(pat) {
          let pos = file.len() - pat.len();
          pos >= 1 && &file[pos-1..pos] == "/"
        } else { false }
      )
    })
  };

  // FIXME: pacman.conf order
  for entry in std::fs::read_dir("/var/lib/pacman/sync")? {
    let entry = entry?;
    let path = entry.path();
    if path.extension() != Some(OsStr::new("files")) {
      continue;
    }
    let repo = path.file_stem().unwrap().to_str().unwrap();

    if args.list {
      list::list_packages(&path, repo, &args.queries)?;
    } else {
      query_files::query_files(&path, repo, &matcher)?;
    }
  }

  Ok(())
}

