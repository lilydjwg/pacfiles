use std::path::Path;
use std::io::IsTerminal;

use tracing_subscriber::EnvFilter;
use rusqlite::{Connection, Error};
use expanduser::expanduser;
use clap::Parser;

mod convert;

#[derive(clap::Parser)]
struct Args {
  #[arg(long, default_value="~/.cache/pacfiles/pacfiles.db")]
  /// The converted database path.
  mydbpath: String,

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

  let path = expanduser(args.mydbpath)?;
  let mut db = setup_db(&path, args.regex)?;

  convert::may_convert_files("/var/lib/pacman/sync", &mut db)?;
  Ok(())
}

fn setup_db(path: &Path, need_regexp: bool) -> eyre::Result<Connection> {
  if let Some(p) = path.parent() {
    std::fs::create_dir_all(p)?;
  }
  let db = Connection::open(path)?;
  if need_regexp {
    add_regexp_function(&db)?;
  }

  db.execute_batch(r"
  CREATE TABLE IF NOT EXISTS repoinfo (
    repo TEXT PRIMARY KEY,
    mtime INTEGER NOT NULL
  );

  CREATE TABLE IF NOT EXISTS files (
    repo TEXT NOT NULL,
    pkgname TEXT NOT NULL,
    path TEXT NOT NULL
  );
  ")?;

  Ok(db)
}

fn add_regexp_function(db: &Connection) -> Result<(), Error> {
  use regex::Regex;
  use rusqlite::functions::FunctionFlags;
  use std::sync::Arc;
  type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

  db.create_scalar_function(
    "regexp",
    2,
    FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
    move |ctx| {
      assert_eq!(ctx.len(), 2, "called with unexpected number of arguments");
      let regexp: Arc<Regex> = ctx.get_or_create_aux(0, |vr| -> Result<_, BoxError> {
        Ok(Regex::new(vr.as_str()?)?)
      })?;
      let is_match = {
        let text = ctx
          .get_raw(1)
          .as_str()
          .map_err(|e| Error::UserFunctionError(e.into()))?;

          regexp.is_match(text)
      };

      Ok(is_match)
    },
  )
}
