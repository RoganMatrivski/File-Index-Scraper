// #![allow(dead_code)]
use walker::walker_async;

mod enums;
mod filters;
mod init;
mod simple_file_info;
mod walker;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use globset::{Glob, GlobMatcher};
    use regex::Regex;

    let args = crate::init::initialize();

    let url = args.url;
    let base = args.base_path + "/";

    let regex_vec = args
        .regex
        .iter()
        .map(|x| Regex::new(x))
        .collect::<Result<Vec<Regex>, regex::Error>>()?;
    let glob_vec = args
        .glob
        .iter()
        .map(|x| Glob::new(x))
        .collect::<Result<Vec<Glob>, globset::Error>>()?
        .iter()
        .map(|x| x.compile_matcher())
        .collect::<Vec<GlobMatcher>>();
    let filters = filters::Filters::new(regex_vec, glob_vec);

    // Get the position of the question mark if exist, else just return string length
    let url_query_startpos = url.chars().position(|x| x == '?').unwrap_or(url.len());
    let (url, url_query) = url.split_at(url_query_startpos);

    // If the url query is just '?', replace with empty string
    let url_query = if url_query == "?" { "" } else { url_query };

    // Appends slash to url if it isn't already
    let url = if !url.ends_with('/') {
        url.to_string() + "/"
    } else {
        url.to_string()
    };

    let res = walker_async(&url, url_query, "".to_string(), &args.sort).await?;

    // Filters by regex
    let res = res
        .iter()
        .filter(|x| {
            let filename = x.get_decoded_filename();

            filters.match_all_regex(&filename)
        })
        .collect::<Vec<&simple_file_info::SimpleFileInfo>>();

    // Filters by glob
    let res = res
        .iter()
        .filter(|x| {
            let path = x.get_decoded_full_path();

            filters.match_all_glob(&path)
        })
        .collect::<Vec<&&simple_file_info::SimpleFileInfo>>();

    // Removes url_query if no_query toggle is true
    // Helps removing clutter if url query is not needed
    let url_query = if args.no_query { "" } else { url_query };

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
        crate::enums::FormatArgs::Link => {
            for link in res {
                println!("{url}{}{}", link.get_decoded_full_path(), url_query);
            }
        }
    }

    Ok(())
}
