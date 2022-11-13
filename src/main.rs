// #![allow(dead_code)]
use clap::{Parser, ValueEnum};
use tracing_subscriber::{fmt, prelude::*, registry, EnvFilter};
use walker::walker_async;

mod simple_file_info;
mod walker;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum FormatArgs {
    PlainText,
    Aria2c,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    url: String,

    #[arg(short, long, value_enum, default_value_t = FormatArgs::PlainText)]
    format: FormatArgs,

    #[arg(short, long, default_value = ".")]
    base_path: String,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

const VERBOSE_LEVEL: &'static [&'static str] = &["info", "debug", "trace"];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    // construct a layer that prints formatted traces to stderr
    let fmt_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_level(true) // include levels in formatted output
        .with_target(true) // include targets
        .with_thread_ids(true) // include the thread ID of the current thread
        .with_thread_names(true); // include the name of the current thread

    registry().with(fmt_layer).with(env_filter).init();

    let url = args.url;
    let base = args.base_path + "/";
    let res = walker_async(url.clone(), base).await?;

    match args.format {
        FormatArgs::PlainText => {
            for link in res {
                println!("{}", link.get_decoded_full_path());
            }
        }
        FormatArgs::Aria2c => {
            for link in res {
                println!(
                    "{}\n    out={}",
                    url.clone() + &link.get_url_relative_path(),
                    link.get_decoded_full_path()
                );
            }
        }
    }

    Ok(())
}
