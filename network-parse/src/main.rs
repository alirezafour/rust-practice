use clap::{Parser, ValueEnum};
use futures::future::join_all;
use network_parse::{
    detect::detect,
    error::FeedError,
    fetch::{FeedFetcher, RequestFetcher},
    parser::{FeedParser, atom::AtomParser, rss::RssParser},
    serializer::{FeedSerializer, atom::AtomSerializer, json::JsonSerializer, rss::RssSerializer},
};

#[derive(Parser)]
#[command(version, about = "RSS/Atom feed aggregator")]
struct Cli {
    urls: Vec<String>,

    #[arg(short, long, default_value_t = 10)]
    limit: usize,

    /// Output format.
    #[arg(short, long, value_enum, default_value_t = Format::Json)]
    format: Format,
}

/// CLI-selectable output format
#[derive(Clone, ValueEnum)]
enum Format {
    Json,
    Rss,
    Atom,
}

impl Format {
    fn serializer(&self) -> Box<dyn FeedSerializer> {
        match self {
            Format::Json => Box::new(JsonSerializer),
            Format::Rss => Box::new(RssSerializer),
            Format::Atom => Box::new(AtomSerializer),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let fetcher = RequestFetcher;

    let tasks = cli.urls.into_iter().map(|url| {
        let fetcher = fetcher.clone();
        let format = cli.format.clone();
        tokio::spawn(async move { process_feed(&fetcher, &url, cli.limit, format).await })
    });

    let results = join_all(tasks).await;

    for result in results {
        match result {
            Ok(Ok(out)) => println!("{out}"),
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
    format: Format,
) -> Result<String, FeedError> {
    let bytes = fetcher.fetch(url).await?;
    let parsed_format = detect(&bytes);
    let mut feed = match parsed_format {
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

    // Apply entry limit before serializing the whole feed.
    feed.entries.truncate(limit.min(feed.entries.len()));

    let out = format.serializer().serialize(&feed)?;
    Ok(String::from_utf8_lossy(&out).into_owned())
}
