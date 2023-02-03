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
    let mut split_url = url.split('?');
    let url = split_url.next().unwrap_or(&url).to_string();
    let url_query = ("?".to_string() + split_url.next().unwrap_or("")).to_string();
    let res = walker_async(&url, &url_query, "".to_string()).await?;

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
                    url.clone() + &link.get_full_path() + &url_query,
                    base.to_owned() + &link.get_decoded_full_path()
                );
            }
        }
    }

    Ok(())
}
