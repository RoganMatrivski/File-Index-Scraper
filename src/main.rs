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
    let res = walker_async(url.clone(), base).await?;

    match args.format {
        crate::enums::FormatArgs::PlainText => {
            for link in res {
                println!("{}", link.get_decoded_full_path());
            }
        }
        crate::enums::FormatArgs::Aria2c => {
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
