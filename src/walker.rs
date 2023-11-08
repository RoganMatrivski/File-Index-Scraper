use anyhow::{bail, Result};
use futures::{stream::FuturesOrdered, StreamExt};

use crate::simple_file_info::SimpleFileInfo;

#[tracing::instrument(skip(url_root), name = "walker")]
// #[async_recursion::async_recursion(?Send)]
#[async_recursion::async_recursion]
pub async fn walker_async<'a>(
    url_root: &'a str,
    url_query: &'a str,
    folder_root: String,
    sort: Option<&'a crate::enums::Sort>,
) -> Result<Vec<SimpleFileInfo>> {
    tracing::info!("Scanning directory");

    if !url_root.ends_with('/') {
        bail!("URL doesn't end with slash ('/')!");
    }

    tracing::debug!("Fetching index HTML file");
    let html: String =
        get_html_async(format!("{url_root}{folder_root}{url_query}").as_str()).await?;

    // This one is overly complicated
    // And i will probably refactor this in the (not so distant) future
    let valid_links = crate::parser::html_to_valid_links(&html, sort)?;

    let dirs = valid_links
        .iter()
        .filter(|&x| x.ends_with('/'))
        .collect::<Vec<&String>>();

    let files = valid_links
        .iter()
        .filter(|&x| !x.ends_with('/'))
        .collect::<Vec<&String>>();

    if !dirs.is_empty() {
        tracing::trace!("Iterating through directories")
    };

    let dir_walker_tasks: FuturesOrdered<_> = dirs
        .iter()
        .map(|dir| walker_async(url_root, url_query, format!("{folder_root}{dir}"), sort))
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
        .collect::<Result<Vec<Vec<SimpleFileInfo>>, anyhow::Error>>()? // Bubble error
        .into_iter()
        .flatten()
        .chain(fileinfos.into_iter())
        .collect::<Vec<SimpleFileInfo>>();

    Ok(res)
}

#[tracing::instrument]
async fn get_html_async(url_str: &str) -> Result<String, reqwest::Error> {
    tracing::trace!("Sending request");
    reqwest::get(url_str).await?.text().await
}
