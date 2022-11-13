// #![allow(dead_code)]
use clap::{Parser, ValueEnum};
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
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let url = args.url;
    let base = args.base_path + "/";
    let res = walker_async(url.clone(), base).await.unwrap();

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
}
