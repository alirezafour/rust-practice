use clap::Parser;
use futures::future::join_all;
use network_parse::{
    detect::detect,
    error::FeedError,
    fetch::{FeedFetcher, RequestFetcher},
    parser::{FeedParser, atom::AtomParser, rss::RssParser},
};

#[derive(Parser)]
#[command(version, about = "RSS/Atom feed aggregator")]
struct Cli {
    urls: Vec<String>,

    #[arg(short, long, default_value_t = 10)]
    limit: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let fetcher = RequestFetcher;

    let tasks = cli.urls.into_iter().map(|url| {
        let fetcher = fetcher.clone();
        tokio::spawn(async move { process_feed(&fetcher, &url, cli.limit).await })
    });

    let results = join_all(tasks).await;

    for result in results {
        match result {
            Ok(Ok(json)) => println!("{json}"),
            Ok(Err(e)) => eprintln!("Feed error: {e}"),
            Err(e) => eprintln!("Task error: {e}"),
        }
    }
    Ok(())
}

async fn process_feed(
    fetcher: &RequestFetcher,
    url: &str,
    limit: usize,
) -> Result<String, FeedError> {
    let bytes = fetcher.fetch(url).await?;
    let format = detect(&bytes);
    let feed = match format {
        network_parse::detect::FeedFormat::Rss => {
            RssParser.parse(&String::from_utf8_lossy(&bytes))?
        }
        network_parse::detect::FeedFormat::Atom => {
            AtomParser.parse(&String::from_utf8_lossy(&bytes))?
        }
        network_parse::detect::FeedFormat::Unknown => {
            return Err(FeedError::Parse(network_parse::error::ParserError {
                tag: "unknown".into(),
                message: "Unknown feed type".into(),
            }));
        }
    };
    Ok(
        serde_json::to_string_pretty(&feed.entries.into_iter().take(limit).collect::<Vec<_>>())
            .unwrap_or_default(),
    )
}
