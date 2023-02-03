use anyhow::{bail, Context, Result};
use futures::{stream::FuturesOrdered, StreamExt};

use crate::simple_file_info::SimpleFileInfo;

const EXCLUDED_CHARS: [char; 2] = ['/', '?'];
const EXCLUDED_PATHS: [&str; 1] = ["../"];

#[tracing::instrument(skip(url_root), name = "walker")]
#[async_recursion::async_recursion(?Send)]
pub async fn walker_async<'a>(
    url_root: &'a str,
    url_query: &'a str,
    folder_root: String,
) -> Result<Vec<SimpleFileInfo>> {
    tracing::info!("Scanning directory");

    if !url_root.ends_with('/') {
        bail!("URL doesn't end with slash ('/')!");
    }

    tracing::debug!("Fetching index HTML file");
    let html: String =
        get_html_async(format!("{url_root}{folder_root}{url_query}").as_str()).await?;

    tracing::trace!("Parsing HTML");
    let dom = tl::parse(&html, tl::ParserOptions::default())?;
    let parser = dom.parser();

    tracing::trace!("Parsing links");
    let element_find = dom
        .query_selector("a[href]")
        .context("Failed to get link element")?;

    let links = element_find
        .into_iter()
        .map(|x| get_href_attr(&x, parser))
        .collect::<Result<Vec<String>>>()?;

    let valid_links = links
        .iter()
        .filter(|&x| is_link_valid(x))
        .collect::<Vec<&String>>();

    let dirs = valid_links
        .iter()
        .filter(|&&x| x.ends_with('/'))
        .collect::<Vec<&&String>>();

    let files = valid_links
        .iter()
        .filter(|&&x| !x.ends_with('/'))
        .collect::<Vec<&&String>>();

    if !dirs.is_empty() {
        tracing::trace!("Iterating through directories")
    };

    let dir_walker_tasks: FuturesOrdered<_> = dirs
        .iter()
        .map(|dir| walker_async(url_root, url_query, format!("{folder_root}{dir}")))
        .collect();

    let dir_walker_results: Vec<_> = dir_walker_tasks.collect().await;

    tracing::trace!("Iterating through files");
    let fileinfos = files
        .iter()
        .map(|x| SimpleFileInfo::new(folder_root.clone(), x.to_string()))
        .collect::<Vec<SimpleFileInfo>>();

    tracing::trace!(total_result = fileinfos.len(), "Appending files");
    let res = dir_walker_results
        .into_iter()
        .collect::<Result<Vec<Vec<SimpleFileInfo>>, anyhow::Error>>()?
        .into_iter()
        .flatten()
        .chain(fileinfos.into_iter())
        .collect::<Vec<SimpleFileInfo>>();

    Ok(res)
}

fn is_link_valid(url: &str) -> bool {
    // Check if link is relative
    // For now, don't check the host. Just check if it's relative or not.
    {
        let lowercased_url = url.to_lowercase();
        let Ok(regex) = regex::Regex::new(r"^(?:[a-z+]+:)?//") else {
            return false;
        };
        if regex.is_match(&lowercased_url) {
            return false;
        }
    }

    // Filter out links beginning with slash or question mark
    {
        let Some(link_first_char) = &url
            .chars()
            .next() else {
                return false
            };

        if EXCLUDED_CHARS.contains(link_first_char) {
            return false;
        }
    }

    // Check if url is in a list of excluded paths
    {
        for path in EXCLUDED_PATHS {
            if url == path {
                // It's excluded, skip
                return false;
            }
        }
    }

    true
}

#[allow(dead_code)]
#[tracing::instrument]
fn walker(url_root: &str, folder_root: &str) -> Result<Vec<SimpleFileInfo>> {
    println!("# Walking through {}", folder_root);

    let new_url = if !url_root.ends_with('/') {
        format!("{}/", url_root)
    } else {
        url_root.to_string()
    };
    let url_root = new_url.as_str();

    let html: String = get_html(format!("{}{}", url_root, folder_root).as_str())?;

    let dom = tl::parse(&html, tl::ParserOptions::default())?;
    let parser = dom.parser();

    let mut dirs: Vec<String> = vec![];
    let mut files: Vec<String> = vec![];

    let element_find = dom
        .query_selector("a[href]")
        .context("Failed to get link element")?;

    for link in element_find {
        let txt = get_href_attr(&link, parser)?;

        if txt.ends_with('/') {
            dirs.push(txt);
        } else {
            files.push(txt);
        }
    }

    let mut paths: Vec<SimpleFileInfo> = vec![];

    for link in dirs {
        let mut res = walker(url_root, format!("{}{}", folder_root, link).as_str())?;

        paths.append(&mut res);
    }

    let mut fileinfos: Vec<SimpleFileInfo> = files
        .iter()
        .map(|x| SimpleFileInfo::new(folder_root.to_string(), x.to_string()))
        .collect();

    paths.append(&mut fileinfos);

    Ok(paths)
}

fn get_href_attr<'a>(
    node_handle: &'a tl::NodeHandle,
    parser: &'a tl::Parser<'a>,
) -> Result<String> {
    let element = node_handle.get(parser).context("Failed to get element")?;
    let tag = element.as_tag().context("Failed to get element as tag")?;
    let attr = tag
        .attributes()
        .get("href")
        .flatten()
        .context("Failed to get href attribute")?;

    Ok(attr.as_utf8_str().to_string())
}

#[tracing::instrument]
async fn get_html_async(url_str: &str) -> Result<String, reqwest::Error> {
    tracing::trace!("Sending request");
    reqwest::get(url_str).await?.text().await
}

#[allow(dead_code)]
fn get_html(url_str: &str) -> Result<String, reqwest::Error> {
    reqwest::blocking::get(url_str)?.text()
}
