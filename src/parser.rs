use anyhow::{Context, Result};
use itertools::Itertools;

const EXCLUDED_CHARS: [char; 2] = ['/', '?'];
const EXCLUDED_PATHS: [&str; 1] = ["../"];

#[tracing::instrument(skip_all)]
pub fn html_to_valid_links<'a>(
    html_str: &'a str,
    sort: Option<&'a crate::enums::Sort>,
) -> Result<Vec<String>> {
    tracing::trace!("Parsing HTML");
    let dom = tl::parse(html_str, tl::ParserOptions::default())?;
    let parser = dom.parser();

    tracing::trace!("Parsing links");
    let element_find = dom
        .query_selector("a[href]")
        .context("Failed to get link element")?;

    let links = element_find
        .into_iter()
        .map(|x| get_href_attr(&x, parser))
        .collect::<Result<Vec<String>>>()?;

    let valid_links_iter = links.into_iter().filter(|x| is_link_valid(x));

    let valid_links = match sort {
        Some(sort_mode) => valid_links_iter
            .sorted_by(|a, b| match sort_mode {
                crate::enums::Sort::Up => Ord::cmp(a, b),
                crate::enums::Sort::Down => Ord::cmp(b, a),
            })
            .collect::<Vec<String>>(),
        None => valid_links_iter.collect::<Vec<String>>(),
    };

    Ok(valid_links)
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
        let Some(link_first_char) = &url.chars().next() else {
            return false;
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
