#![allow(dead_code)]
use clap::{Parser, ValueEnum};
use tl;

#[derive(Debug)]
struct SimpleFileInfo {
    dir: String,
    file: String,
}

impl SimpleFileInfo {
    fn get_full_path(&self) -> String {
        format!("{}{}", self.dir, self.file)
    }

    fn get_url_relative_path(&self) -> String {
        self.get_full_path()[2..].to_string()
    }

    fn get_decoded_full_path(&self) -> String {
        urlencoding::decode(&self.get_full_path())
            .unwrap()
            .into_owned()
    }

    fn new(_dir: String, _file: String) -> SimpleFileInfo {
        SimpleFileInfo {
            dir: _dir,
            file: _file,
        }
    }
}

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

fn main() {
    let args = Args::parse();

    let url = args.url;
    let base = args.base_path + "/";
    let res = walker(&url, &base).unwrap();


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

fn get_html(url_str: &str) -> Result<String, reqwest::Error> {
    reqwest::blocking::get(url_str)?.text()
}
