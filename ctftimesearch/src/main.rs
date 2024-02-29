use async_channel::{bounded, unbounded};
use indicatif::ProgressBar;
use mdka::from_html;
use once_cell::sync::Lazy;
use reqwest::{StatusCode, ClientBuilder};
use scraper::{Html, Selector};
use std::{sync::mpsc, fs::File};
use std::thread;
use std::io::{Read, BufReader, BufRead, BufWriter, Write};
use tantivy::{
    doc,
    query::{EnableScoring, QueryParser},
    schema::{Field, FieldType, Schema, STORED, TEXT},
    Document, Index, TERMINATED,
};
use termimad::text;
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
    // #[error("Failed to parse tags")]
    // TagParsing,
    #[error("Failed to parse author")]
    AuthorParsing,
    // #[error("Failed to parse team")]
    // TeamParsing,
    #[error("Failed to parse event")]
    EventParsing,
    #[error("Failed to parse title")]
    TitleParsing,
    #[error("Failed to parse original writeup link")]
    WriteupLinkParsing,
    #[error("Failed to parse description")]
    DescriptionParsing,
    #[error("Failed to parse original description")]
    OriginalDescriptionParsing,
    #[error("Page doesn't exist")]
    NoSuchPage,
    #[error("Request Error: $1")]
    Request(#[from] reqwest::Error),
}

#[derive(serde::Serialize)]
struct Writeup {
    pub title: String,
    pub description: String,
    pub author: String,
    pub team: Option<String>,
    pub tags: Vec<String>,
    pub event: String,
    pub orig_writeup_link: Option<String>,
    pub link: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let threads = 8;
    let starting_page = 30000;
    let ending_page = 38715;
    let buffer_size = 8000_000_000;

    index(threads, starting_page, ending_page, buffer_size, "./index")?;

    // let reader = index.reader()?;
    // let searcher = reader.searcher();
    // let query_parser = QueryParser::for_index(&index, vec![title, description, tags]);
    //
    // let query = query_parser.parse_query("hacking")?;
    //
    // let top_docs: Vec<(Score, DocAddress)> =
    //     searcher.search(&query, &TopDocs::with_limit(10))?;
    // for (_score, doc_address) in top_docs {
    //     let retrieved_doc = searcher.doc(doc_address)?;
    //     println!("{:?}",schema.to_json(&retrieved_doc));
    // }
    Ok(())
}

fn parse_page(body: &str, link: &str) -> Result<Writeup, Error> {
    let parsed_body = Html::parse_document(&body);

    // let four_o_four_selector = Selector::parse(".span10 > h1:nth-child(1)").unwrap();
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

    // let body = parsed_body.select(&BODY_SELECTOR).next().ok_or(Error::DescriptionParsing)?.inner_html();
    let description = match parsed_body.select(&BODY_SELECTOR).next() {
        Some(body) if !link_in_body => from_html(&body.inner_html()),
        _ => {
            let link = orig_writeup_link.clone().ok_or(Error::OriginalDescriptionParsing)?;
            if link.contains("://github.com/") {
                let repo_path = link.split_once("://github.com/").ok_or(Error::OriginalDescriptionParsing)?.1;
                let link = format!(
                    "https://raw.githubusercontent.com/{}/README.md",
                    repo_path
                );
                let link = link.replace("/tree/","/");
                println!("{}",link);
                match reqwest::blocking::get(link) {
                    Ok(result) => 
                        {
                            if !result.status().is_success() {
                                return Err(Error::OriginalDescriptionParsing);
                            }
                            match result.text() {
                            Ok(text) => text,
                            Err(error) => {
                                log::error!(target:"ctftimesearch::fetcher","{}",error);
                                return Err(Error::OriginalDescriptionParsing);
                            }
                        }
                    },
                    Err(error) => {
                        log::error!(target:"ctftimesearch::fetcher","{}",error);
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
        tags,
        event,
        title,
        team,
        orig_writeup_link,
        link: link.to_string(),
    })
}

fn search<P: AsRef<std::path::Path>>(
    index_path: P,
    query: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let index = Index::open_in_dir(index_path)?;
    let schema = index.schema();
    let default_fields: Vec<Field> = schema
        .fields()
        .filter(|&(_, ref field_entry)| match *field_entry.field_type() {
            FieldType::Str(ref text_field_options) => {
                text_field_options.get_indexing_options().is_some()
            }
            _ => false,
        })
        .map(|(field, _)| field)
        .collect();
    let query_parser = QueryParser::new(schema.clone(), default_fields, index.tokenizers().clone());
    let query = query_parser.parse_query(query)?;
    let searcher = index.reader()?.searcher();
    let weight = query.weight(EnableScoring::enabled_from_searcher(&searcher))?;
    for segment_reader in searcher.segment_readers() {
        let mut scorer = weight.scorer(segment_reader, 1.0)?;
        let store_reader = segment_reader.get_store_reader(100)?;
        while scorer.doc() != TERMINATED {
            let doc_id = scorer.doc();
            let doc: Document = store_reader.get(doc_id)?;
            let named_doc = schema.to_named_doc(&doc);
            let author = named_doc
                .0
                .get("author")
                .unwrap()
                .get(0)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string();
            let event = named_doc
                .0
                .get("event")
                .unwrap()
                .get(0)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string();
            let link = named_doc
                .0
                .get("link")
                .unwrap()
                .get(0)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string();
            let description = named_doc
                .0
                .get("description")
                .unwrap()
                .get(0)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string();
            let original_writeup_link = named_doc
                .0
                .get("original_writeup_link")
                .unwrap()
                .get(0)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string();
            let title = named_doc
                .0
                .get("title")
                .unwrap()
                .get(0)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string();
            let link = named_doc
                .0
                .get("link")
                .unwrap()
                .get(0)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string();
            let tags: Vec<String> = named_doc
                .0
                .get("tags")
                .unwrap()
                .iter()
                .map(|el| el.as_text().unwrap().to_string())
                .collect();
        }
    }
    Ok(())
}

fn index<P: AsRef<std::path::Path>>(
    threads: usize,
    starting_page: u64,
    ending_page: u64,
    buffer_size: usize,
    index_path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut schema_builder = Schema::builder();
    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let description = schema_builder.add_text_field("description", TEXT | STORED);
    let author = schema_builder.add_text_field("author", TEXT | STORED);
    let team = schema_builder.add_text_field("team", TEXT | STORED);
    let tags = schema_builder.add_text_field("tags", TEXT | STORED);
    let event = schema_builder.add_text_field("event", TEXT | STORED);
    let orig_writeup_link = schema_builder.add_text_field("original_writeup_link", TEXT | STORED);
    let link = schema_builder.add_text_field("link", TEXT | STORED);

    let schema = schema_builder.build();

    // let mut index = Index::create_in_dir(index_path, schema.clone())?;
    // index.set_multithread_executor(threads)?;

    let (page_number_sender, page_number_reciever) = unbounded();

    log::info!("spawning page number green thread");
    {
        // let page_number_sender = page_number_sender.clone();
        tokio::spawn(async move {
            for i in starting_page..ending_page {
                match page_number_sender.send(i).await {
                    Ok(()) => {}
                    Err(error) => log::error!("{}", error),
                }
            }
        });
    }

    let (sender, reciever) = bounded(100);

    log::info!("spawning {} page fetching green threads", threads);

    // let proxies_file = File::open("./good.txt")?;
    // let buf = BufReader::new(proxies_file);
    // let mut proxies: Vec<String> = buf.lines().map(|l| l.expect("Could not parse line")).collect();
    //
    // for _ in 0..(threads*5) {
    //     // let page_number_sender = page_number_sender.clone();
    //     let sender = sender.clone();
    //     let page_number_reciever = page_number_reciever.clone();
    //     let proxy = proxies.pop().unwrap();
    //     let client = ClientBuilder::new().proxy(reqwest::Proxy::http(format!("http://{}",proxy))?).build()?;

    //     let proxies_file = File::open("./proxies.txt")?;
    // let buf = BufReader::new(proxies_file);
    // let mut proxies: Vec<(String, String)> = buf
    //     .lines()
    //     .into_iter()
    //     .map(|l| {
    //         let string = l.unwrap();
    //         let (str1, str2) = string.split_once("@").unwrap();
    //         (format!("https://{}",str1), str2.to_string())
    //     })
    //     .collect();

    for _ in 0..4 {
        // let page_number_sender = page_number_sender.clone();
        let sender = sender.clone();
        let page_number_reciever = page_number_reciever.clone();
        // let (proxy, creds) = proxies.pop().unwrap();
        // let (username, password) = creds.split_once(":").unwrap();
        let client = ClientBuilder::new()
            // .proxy(reqwest::Proxy::http(proxy)?.basic_auth(username, password))
            .build()?;

        tokio::spawn(async move {
            while let Ok(page_number) = page_number_reciever.recv().await {
                let link = format!("https://ctftime.org/writeup/{}", page_number);
                log::debug!("parsing page {}", link);
                let request = match client.get(&link).send().await {
                    Ok(request) => request,
                    Err(error) => {
                        log::error!(target:"ctftimesearch::fetcher","{}",error);
                        continue;
                    }
                };
                if !request.status().is_success() && request.status() != StatusCode::NOT_FOUND {
                    log::error!(target:"ctftimesearch::fetcher","caught rate limit?");
                    // match page_number_sender.send(page_number).await {
                    //     Ok(()) => {},
                    //     Err(error) => log::error!("{}",error),
                    // };
                    tokio::time::sleep(tokio::time::Duration::from_millis(10000)).await;
                    continue;
                };
                let body = match request.text().await {
                    Ok(body) => body,
                    Err(error) => {
                        log::error!(target:"ctftimesearch::fetcher","{}",error);
                        continue;
                    }
                };
                match sender.send((page_number, link, body)).await {
                    Ok(()) => {}
                    Err(error) => log::error!(target:"ctftimesearch::fetcher","{}",error),
                }
            }
        });
    }

    drop(sender);
    drop(page_number_reciever);

    let (document_sender, document_reciever) = mpsc::channel();

    log::info!(
        "spawning {} page parsing os threads",
        std::cmp::max(1, threads / 4)
    );

    for _ in 0..8 {
        // let page_number_sender = page_number_sender.clone();
        let document_sender = document_sender.clone();
        let reciever = reciever.clone();
        thread::spawn(move || {
            while let Ok((_page_number, writeup_link, body)) = reciever.recv_blocking() {
                let writeup = match parse_page(&body, &writeup_link) {
                    Ok(writeup) => writeup,
                    Err(Error::AuthorParsing) => {
                        log::error!(target:"ctftimesearch::parser","{} : probably caught rate limit",writeup_link);
                        // match page_number_sender.send_blocking(page_number) {
                        //     Ok(()) => {},
                        //     Err(error) => log::error!("{}",error),
                        // };
                        continue;
                    }
                    Err(error) => {
                        log::error!(target:"ctftimesearch::parser","{} : {}",writeup_link,error);
                        continue;
                    }
                };
                let writeup_json = serde_json::to_string(&writeup).unwrap();
                // let mut document = doc!(
                //     title => writeup.title,
                //     description => writeup.description,
                //     author => writeup.author,
                //     event => writeup.event,
                //     link => writeup.link,
                // );
                // if let Some(team_name) = writeup.team {
                //     document.add_text(team, team_name);
                // }
                // if let Some(link) = writeup.orig_writeup_link {
                //     document.add_text(orig_writeup_link, link);
                // }
                // for tag in writeup.tags {
                //     document.add_text(tags, tag);
                // }
                match document_sender.send(writeup_json) {
                    Ok(()) => {}
                    Err(error) => log::error!(target:"ctftimesearch::parser","{}",error),
                };
            }
        });
    }
    drop(reciever);
    drop(document_sender);

    // log::info!("creating index writer with {} threads", threads);
    //
    // let mut index_writer = index.writer_with_num_threads(threads, buffer_size)?;
    //
    // log::info!("starting indexing");
    //
    let bar = ProgressBar::new(ending_page - starting_page);
    let file = File::create("./doc3.json").unwrap();
    let mut file = BufWriter::new(file);

    while let Ok(document) = document_reciever.recv() {

        // index_writer.add_document(document)?;
        file.write_all(document.as_bytes()).unwrap();
        bar.inc(1);
    }
    bar.finish();
    log::info!("finished indexing, committing");
    // let _res = index_writer.commit()?;
    Ok(())
}
