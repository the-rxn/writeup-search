use async_channel::{bounded, unbounded};
use clap::Parser;
use indicatif::ProgressBar;
use mdka::from_html;
use once_cell::sync::Lazy;
use reqwest::{ClientBuilder, StatusCode};
use scraper::{Html, Selector};
use std::io::{BufWriter, Write, Seek};
use std::path::PathBuf;
use std::thread;
use std::{fs::File, sync::mpsc};
use thiserror::Error;

static FOUR_O_FOUR_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".span10 > h1:nth-child(1)").unwrap());
static TAG_DIV_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".span7 > p:nth-child(1)").unwrap());
static TAG_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("span.label").unwrap());
static AUTHOR_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.page-header:nth-child(1) > a:nth-child(2)").unwrap());
static TEAM_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.page-header:nth-child(1) > a:nth-child(3)").unwrap());
static EVENT_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".breadcrumb > li:nth-child(3) > a:nth-child(1)").unwrap());
static TITLE_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("html body div.container div.page-header h2").unwrap());
static ORIG_WRITEUP_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.well:nth-child(2) > a:nth-child(1)").unwrap());
static BODY_WRITEUP_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("#id_description > p:nth-child(1) > a:nth-child(1)").unwrap());
static BODY_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("#id_description").unwrap());

#[derive(Error, Debug)]
enum Error {
    #[error("Failed to parse author")]
    AuthorParsing,
    #[error("Failed to parse event")]
    EventParsing,
    #[error("Failed to parse title")]
    TitleParsing,
    #[error("Failed to parse original writeup link")]
    WriteupLinkParsing,
    #[error("Failed to parse original description")]
    OriginalDescriptionParsing,
    #[error("Page doesn't exist")]
    NoSuchPage,
    #[error("Request Error: $1")]
    Request(#[from] reqwest::Error),
    // #[error("Io Error: $1")]
    // Io(#[from] std::io::Error),
}

#[derive(serde::Serialize)]
struct Writeup {
    pub title: String,
    pub description: String,
    pub author: String,
    pub team: String,
    pub tags: String,
    pub event: String,
    pub orig_writeup_link: String,
    pub link: String,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// path to output json file
    #[arg(short, long)]
    output: PathBuf,
    /// index of writeup to start parsing from
    #[arg(short, long)]
    start: u64,
    /// index of writeup to stop parsing at
    #[arg(short, long)]
    end: u64,
    /// number of fetching threads (if you start seeing rate limit errors decrease thir number)
    #[arg(short, long, default_value_t = 4)]
    fetching_threads: usize,
    /// number of parsing threads
    #[arg(short,long,default_value_t = num_cpus::get())]
    parsing_threads: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();

    parse(
        args.parsing_threads,
        args.fetching_threads,
        args.start,
        args.end,
        args.output,
    )?;

    Ok(())
}

fn parse_page(body: &str, link: &str) -> Result<Writeup, Error> {
    let parsed_body = Html::parse_document(&body);

    if let Some(tag) = parsed_body.select(&FOUR_O_FOUR_SELECTOR).next() {
        if &tag.inner_html() == "404" {
            return Err(Error::NoSuchPage);
        }
    }

    let tags: Vec<String> = match parsed_body.select(&TAG_DIV_SELECTOR).next() {
        Some(tags_div) => tags_div
            .select(&TAG_SELECTOR)
            .map(|tag| tag.inner_html())
            .collect(),
        None => vec![],
    };

    let author = parsed_body
        .select(&AUTHOR_SELECTOR)
        .next()
        .ok_or(Error::AuthorParsing)?
        .inner_html();

    let team = parsed_body
        .select(&TEAM_SELECTOR)
        .next()
        .map(|element| element.inner_html());

    let event = parsed_body
        .select(&EVENT_SELECTOR)
        .next()
        .ok_or(Error::EventParsing)?
        .inner_html();

    let title = parsed_body
        .select(&TITLE_SELECTOR)
        .next()
        .ok_or(Error::TitleParsing)?
        .inner_html();

    let mut link_in_body = false;
    let orig_writeup_link = match parsed_body.select(&ORIG_WRITEUP_SELECTOR).next() {
        Some(element) => Some(
            element
                .attr("href")
                .ok_or(Error::WriteupLinkParsing)?
                .to_string(),
        ),
        None => match parsed_body.select(&BODY_WRITEUP_SELECTOR).next() {
            Some(element) => {
                if !element.inner_html().contains("writeup") {
                    None
                } else {
                    link_in_body = true;
                    element.attr("href").map(|link| link.to_string())
                }
            }
            None => None,
        },
    };

    let description = match parsed_body.select(&BODY_SELECTOR).next() {
        Some(body) if !link_in_body => from_html(&body.inner_html()),
        _ => {
            let link = orig_writeup_link
                .clone()
                .ok_or(Error::OriginalDescriptionParsing)?;
            if link.contains("://github.com/") {
                let repo_path = link
                    .split_once("://github.com/")
                    .ok_or(Error::OriginalDescriptionParsing)?
                    .1;
                let link = format!("https://raw.githubusercontent.com/{}/README.md", repo_path);
                let link = link.replace("/tree/", "/");
                println!("{}", link);
                match reqwest::blocking::get(link) {
                    Ok(result) => {
                        if !result.status().is_success() {
                            return Err(Error::OriginalDescriptionParsing);
                        }
                        match result.text() {
                            Ok(text) => text,
                            Err(error) => {
                                log::debug!(target:"ctftimesearch::fetcher","{}",error);
                                return Err(Error::OriginalDescriptionParsing);
                            }
                        }
                    }
                    Err(error) => {
                        log::debug!(target:"ctftimesearch::fetcher","{}",error);
                        return Err(Error::OriginalDescriptionParsing);
                    }
                }
            } else {
                return Err(Error::OriginalDescriptionParsing);
            }
        }
    };

    Ok(Writeup {
        description,
        author,
        tags: tags.into_iter().fold(String::new(), |mut a, b| {
            a.push_str(&b);
            a
        }),
        event,
        title,
        team: team.unwrap_or("".into()),
        orig_writeup_link: orig_writeup_link.unwrap_or("".into()),
        link: link.to_string(),
    })
}

fn parse(
    parsing_threads: usize,
    fetching_threads: usize,
    starting_page: u64,
    ending_page: u64,
    out_path: PathBuf,
) -> anyhow::Result<()> {
    let (page_number_sender, page_number_reciever) = unbounded();

    log::info!("spawning page number green thread");
    tokio::spawn(async move {
        for i in starting_page..ending_page {
            match page_number_sender.send(i).await {
                Ok(()) => {}
                Err(error) => log::debug!("{}", error),
            }
        }
    });

    let (sender, reciever) = bounded(100);

    log::info!("spawning {} page fetching green threads", fetching_threads);

    for _ in 0..fetching_threads {
        let sender = sender.clone();
        let page_number_reciever = page_number_reciever.clone();
        let client = ClientBuilder::new().build()?;

        tokio::spawn(async move {
            while let Ok(page_number) = page_number_reciever.recv().await {
                let link = format!("https://ctftime.org/writeup/{}", page_number);
                log::debug!("parsing page {}", link);
                let request = match client.get(&link).send().await {
                    Ok(request) => request,
                    Err(error) => {
                        log::debug!(target:"ctftimesearch::fetcher","{}",error);
                        continue;
                    }
                };
                if !request.status().is_success() && request.status() != StatusCode::NOT_FOUND {
                    log::error!(target:"ctftimesearch::fetcher","caught rate limit?");
                    tokio::time::sleep(tokio::time::Duration::from_millis(10000)).await;
                    continue;
                };
                let body = match request.text().await {
                    Ok(body) => body,
                    Err(error) => {
                        log::debug!(target:"ctftimesearch::fetcher","{}",error);
                        continue;
                    }
                };
                match sender.send((link, body)).await {
                    Ok(()) => {}
                    Err(error) => log::debug!(target:"ctftimesearch::fetcher","{}",error),
                }
            }
        });
    }

    drop(sender);
    drop(page_number_reciever);

    let (document_sender, document_reciever) = mpsc::channel();

    log::info!("spawning {} page parsing os threads", parsing_threads);

    for _ in 0..parsing_threads {
        let document_sender = document_sender.clone();
        let reciever = reciever.clone();
        thread::spawn(move || {
            while let Ok((writeup_link, body)) = reciever.recv_blocking() {
                let writeup = match parse_page(&body, &writeup_link) {
                    Ok(writeup) => writeup,
                    Err(Error::AuthorParsing) => {
                        log::error!(target:"ctftimesearch::parser","{} : probably caught rate limit",writeup_link);
                        continue;
                    }
                    Err(error) => {
                        log::debug!(target:"ctftimesearch::parser","{} : {}",writeup_link,error);
                        continue;
                    }
                };
                let writeup_json = match serde_json::to_string(&writeup) {
                    Ok(json) => json,
                    Err(error) => {
                        log::debug!(target:"ctftimesearch::parser", "{}",error);
                        continue;
                    }
                };
                match document_sender.send(writeup_json) {
                    Ok(()) => {}
                    Err(error) => log::debug!(target:"ctftimesearch::parser","{}",error),
                };
            }
        });
    }
    drop(reciever);
    drop(document_sender);

    let bar = ProgressBar::new(ending_page - starting_page);
    let file = File::create(out_path)?;
    let mut file = BufWriter::new(file);

    file.write_all("[".as_bytes())?;

    while let Ok(document) = document_reciever.recv() {
        file.write_all(document.as_bytes())?;
        file.write_all(",".as_bytes())?;
        bar.inc(1);
    }
    file.seek(std::io::SeekFrom::End(-1))?;
    file.write_all("]".as_bytes())?;
    bar.finish();
    log::info!("finished indexing, committing");
    Ok(())
}
