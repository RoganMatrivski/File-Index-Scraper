use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, registry, EnvFilter};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The index URL to query
    pub url: String,

    /// The output format, defaults to printing all the relative URLs in each line
    #[arg(short, long, value_enum, default_value_t = crate::enums::FormatArgs::PlainText)]
    pub format: crate::enums::FormatArgs,

    /// Path to prepend on each `out=` path. Works only on aria2c format
    #[arg(short, long, default_value = ".")]
    pub base_path: String,

    /// Remove query string from output
    #[arg(long)]
    pub no_query: bool,

    /// Sort the result alphabetically
    #[arg(short, long, value_enum)]
    pub sort: Option<crate::enums::Sort>,

    /// Verbosity log
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

const VERBOSE_LEVEL: &[&str] = &["info", "debug", "trace"];

pub fn initialize() -> Args {
    let args = Args::parse();

    let verbosity = match args.verbose {
        1..=3 => Some(VERBOSE_LEVEL[(args.verbose as usize) - 1]),
        _ => None,
    };

    let env_filter = EnvFilter::from_default_env().add_directive(tracing::Level::WARN.into());
    let env_filter = match verbosity {
        Some(v) => env_filter.add_directive(
            format!("{}={}", env!("CARGO_PKG_NAME"), v)
                .parse()
                .expect("Failed to parse log parameter"),
        ),
        None => env_filter,
    };

    // Default logging layer
    let fmt_layer = fmt::layer().with_writer(std::io::stderr);

    match verbosity {
        Some(_) => {
            // construct a layer that prints formatted traces to stderr
            let fmt_layer = fmt_layer
                .with_level(true) // include levels in formatted output
                .with_target(true) // include targets
                .with_thread_ids(true) // include the thread ID of the current thread
                .with_thread_names(true); // include the name of the current thread

            registry().with(fmt_layer).with(env_filter).init();
        }
        None => {
            // construct a layer that prints formatted traces to stderr
            let fmt_layer = fmt_layer.without_time().compact();

            registry().with(fmt_layer).with(env_filter).init();
        }
    };

    args
}
