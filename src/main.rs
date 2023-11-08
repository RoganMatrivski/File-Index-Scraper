// #![allow(dead_code)]
use walker::walker_async;

use crate::json_struct::{JsonFile, JsonList};

mod enums;
mod filters;
mod init;
mod json_struct;
mod simple_file_info;
mod walker;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use globset::{Glob, GlobMatcher};
    use regex::Regex;

    let args = crate::init::initialize();

    let url = args.url;
    let base = {
        // Crude way to make sure the base-path is a directory path
        std::path::PathBuf::from(if !args.base_path.ends_with('/') {
            args.base_path + "/"
        } else {
            args.base_path.to_owned()
        })
    };

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
    let exclude_regex_vec = args
        .exclude_regex
        .iter()
        .map(|x| Regex::new(x))
        .collect::<Result<Vec<Regex>, regex::Error>>()?;
    let exclude_glob_vec = args
        .exclude_glob
        .iter()
        .map(|x| Glob::new(x))
        .collect::<Result<Vec<Glob>, globset::Error>>()?
        .iter()
        .map(|x| x.compile_matcher())
        .collect::<Vec<GlobMatcher>>();
    let filters = filters::Filters::new(regex_vec, glob_vec, exclude_regex_vec, exclude_glob_vec);

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
    let res: Vec<_> = res
        .iter()
        .filter(|x| {
            let filename = x.get_decoded_filename();

            filters.match_all_regex(&filename)
                && filters.match_all_glob(&filename)
                && filters.match_all_regex_exclude(&filename)
                && filters.match_all_glob_exclude(&filename)
        })
        .collect();

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
                let joined_path = base.join(&link.get_decoded_full_path());

                // This shouldn't've been possible. (wow what a word contraction)
                // If it does, someone is going to let me know as soon as i publish this.
                let path_parent = joined_path.parent().unwrap().to_string_lossy();
                let filename = joined_path.file_name().unwrap().to_string_lossy();

                println!(
                    "{}\n\tdir={}\n\tout={}",
                    url.to_owned() + &link.get_full_path() + url_query,
                    path_parent,
                    filename
                );
            }
        }
        crate::enums::FormatArgs::Link => {
            for link in res {
                println!("{url}{}{}", link.get_decoded_full_path(), url_query);
            }
        }
        crate::enums::FormatArgs::Json => {
            let entries = res
                .iter()
                .map(|x| JsonFile {
                    filepath: std::path::PathBuf::from(x.get_full_path()),
                    encoded_filepath: std::path::PathBuf::from(x.get_decoded_full_path()),
                })
                .collect::<Vec<JsonFile>>();

            let json_list = JsonList {
                version: "1.0".to_string(),
                base_url: url.to_string(),
                url_query: url_query.to_string(),
                files: entries,
            };

            let json_encoded = serde_json::to_string_pretty(&json_list)?;

            println!("{json_encoded}")
        }
    }

    Ok(())
}
