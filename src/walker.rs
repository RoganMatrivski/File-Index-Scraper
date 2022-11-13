use futures::stream::FuturesOrdered;
use futures::StreamExt;

use crate::simple_file_info::SimpleFileInfo;

#[async_recursion::async_recursion(?Send)]
pub async fn walker_async(
    url_root: String,
    folder_root: String,
) -> Result<Vec<SimpleFileInfo>, &'static str> {
    println!("# Walking through {}", folder_root);

    let new_url = if !url_root.ends_with("/") {
        format!("{}/", url_root)
    } else {
        url_root.to_string()
    };
    let url_root = new_url;

    let html: String = get_html_async(format!("{}{}", url_root, folder_root).as_str())
        .await
        .unwrap();

    let dom = tl::parse(&html, tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();

    let mut dirs: Vec<String> = vec![];
    let mut files: Vec<String> = vec![];

    for link in dom.query_selector("a[href]").unwrap().into_iter() {
        let txt = get_href_attr(link, parser).unwrap();

        if txt.ends_with('/') {
            dirs.push(txt);
        } else {
            files.push(txt);
        }
    }

    let mut paths: Vec<SimpleFileInfo> = vec![];

    let dir_walker_tasks: FuturesOrdered<_> = dirs
        .into_iter()
        .map(|dir| {
            walker_async(
                url_root.clone(),
                format!("{}{}", folder_root.clone(), dir.clone()),
            )
        })
        .collect();

    let dir_walker_results: Vec<_> = dir_walker_tasks.collect().await;
    for result in dir_walker_results {
        let mut result = result.unwrap();

        paths.append(&mut result);
    }

    let mut fileinfos: Vec<SimpleFileInfo> = files
        .iter()
        .map(|x| SimpleFileInfo::new(folder_root.to_string(), x.to_string()))
        .collect();

    paths.append(&mut fileinfos);

    Ok(paths)
}

#[allow(dead_code)]
fn walker(url_root: &str, folder_root: &str) -> Result<Vec<SimpleFileInfo>, &'static str> {
    println!("# Walking through {}", folder_root);

    let new_url = if !url_root.ends_with("/") {
        format!("{}/", url_root)
    } else {
        url_root.to_string()
    };
    let url_root = new_url.as_str();

    let html: String = get_html(format!("{}{}", url_root, folder_root).as_str()).unwrap();

    let dom = tl::parse(&html, tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();

    let mut dirs: Vec<String> = vec![];
    let mut files: Vec<String> = vec![];

    for link in dom.query_selector("a[href]").unwrap().into_iter() {
        let txt = get_href_attr(link, parser).unwrap();

        if txt.ends_with('/') {
            dirs.push(txt);
        } else {
            files.push(txt);
        }
    }

    let mut paths: Vec<SimpleFileInfo> = vec![];

    for link in dirs {
        let mut res = walker(url_root, format!("{}{}", folder_root, link).as_str()).unwrap();

        paths.append(&mut res);
    }

    let mut fileinfos: Vec<SimpleFileInfo> = files
        .iter()
        .map(|x| SimpleFileInfo::new(folder_root.to_string(), x.to_string()))
        .collect();

    paths.append(&mut fileinfos);

    Ok(paths)
}

fn get_href_attr(
    node_handle: tl::NodeHandle,
    parser: &tl::Parser<'_>,
) -> Result<String, &'static str> {
    let Some(el) = node_handle.get(parser) else {
        return Err("Failed to get Element");
    };

    let Some(el_tag) = el.as_tag() else {
        return Err("Failed to get element as tag");
    };

    let Some(attr) = el_tag.attributes().get("href").flatten() else {
        return Err("Failed to get href attribute");
    };

    Ok(attr.as_utf8_str().to_string())
}

async fn get_html_async(url_str: &str) -> Result<String, reqwest::Error> {
    reqwest::get(url_str).await?.text().await
}

#[allow(dead_code)]
fn get_html(url_str: &str) -> Result<String, reqwest::Error> {
    reqwest::blocking::get(url_str)?.text()
}