// #![allow(dead_code)]
use walker::walker_async;

mod enums;
mod init;
mod simple_file_info;
mod walker;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = crate::init::initialize();

    let url = args.url;
    let base = args.base_path + "/";

    // Get the position of the question mark if exist, else just return string length
    let url_query_startpos = url.chars().position(|x| x == '?').unwrap_or(url.len());
    let (url, url_query) = url.split_at(url_query_startpos);
    let res = walker_async(url, url_query, "".to_string()).await?;

    match args.format {
        crate::enums::FormatArgs::PlainText => {
            for link in res {
                println!("{}{}", link.get_decoded_full_path(), url_query);
            }
        }
        crate::enums::FormatArgs::Aria2c => {
            for link in res {
                println!(
                    "{}\n    out={}",
                    url.to_owned() + &link.get_full_path() + url_query,
                    base.to_owned() + &link.get_decoded_full_path()
                );
            }
        }
    }

    Ok(())
}
