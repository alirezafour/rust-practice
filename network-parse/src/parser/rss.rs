use super::FeedParser;
use crate::{
    error::{FeedError, ParserError},
    model::{Entry, Feed},
};
use chrono::{DateTime, Utc};
use quick_xml::Reader;
use quick_xml::events::Event;

pub struct RssParser;

impl FeedParser for RssParser {
    fn parse(&self, xml: &str) -> Result<Feed, FeedError> {
        let mut reader = Reader::from_str(xml);

        //state - track the current pos in xml
        let mut in_channel = false;
        let mut in_item = false;
        let mut current_tag = String::new();

        // accumulate data
        let mut title = String::new();
        let mut link = String::new();
        let mut description = String::new();
        let mut entries: Vec<Entry> = Vec::new();

        // current item
        let mut item_title = String::new();
        let mut item_link = String::new();
        let mut item_desc = String::new();
        let mut item_published: Option<DateTime<Utc>> = None;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ev)) => {
                    let tag = String::from_utf8_lossy(ev.name().as_ref()).to_string();
                    match tag.as_str() {
                        "channel" => in_channel = true,
                        "item" => in_item = true,
                        _ => current_tag = tag,
                    }
                }
                Ok(Event::Text(ev)) => {
                    let text = String::from_utf8_lossy(ev.as_ref()).to_string();
                    if text.is_empty() {
                        continue;
                    }
                    if in_item {
                        match current_tag.as_str() {
                            "title" => item_title.push_str(&text),
                            "link" => item_link.push_str(&text),
                            "description" => item_desc.push_str(&text),
                            "pubDate" => item_published = parse_rfc2822(&text)?,
                            _ => {}
                        }
                    } else if in_channel {
                        match current_tag.as_str() {
                            "title" => title.push_str(&text),
                            "link" => link.push_str(&text),
                            "description" => description.push_str(&text),
                            _ => {}
                        }
                    }
                }
                Ok(Event::CData(ev)) => {
                    let text = String::from_utf8_lossy(ev.as_ref()).trim().to_string();
                    if text.is_empty() {
                        continue;
                    }
                    if in_item {
                        match current_tag.as_str() {
                            "title" => item_title.push_str(&text),
                            "link" => item_link.push_str(&text),
                            "description" => item_desc.push_str(&text),
                            "pubDate" => item_published = parse_rfc2822(&text)?,
                            _ => {}
                        }
                    } else if in_channel {
                        match current_tag.as_str() {
                            "title" => title.push_str(&text),
                            "link" => link.push_str(&text),
                            "description" => description.push_str(&text),
                            _ => {}
                        }
                    }
                }
                Ok(Event::GeneralRef(ev)) => {
                    let text: String = match String::from_utf8_lossy(ev.as_ref()).trim() {
                        "amp" => "&".into(),
                        "lt" => "<".into(),
                        "gt" => ">".into(),
                        "quot" => "\"".into(),
                        "apos" => "'".into(),
                        _ => "".into(),
                    };
                    if text.is_empty() {
                        continue;
                    }
                    if in_item {
                        match current_tag.as_str() {
                            "title" => item_title.push_str(&text),
                            "link" => item_link.push_str(&text),
                            "description" => item_desc.push_str(&text),
                            "pubDate" => item_published = parse_rfc2822(&text)?,
                            _ => {}
                        }
                    } else if in_channel {
                        match current_tag.as_str() {
                            "title" => title.push_str(&text),
                            "link" => link.push_str(&text),
                            "description" => description.push_str(&text),
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ev)) => {
                    let tag = String::from_utf8_lossy(ev.name().as_ref()).to_string();
                    if tag == "item" && in_item {
                        entries.push(Entry {
                            id: item_link.clone(),
                            title: if item_title.is_empty() {
                                None
                            } else {
                                Some(item_title.trim().into())
                            },
                            link: if item_link.is_empty() {
                                None
                            } else {
                                Some(item_link.trim().into())
                            },
                            summary: if item_desc.is_empty() {
                                None
                            } else {
                                Some(item_desc.trim().into())
                            },
                            content: None,
                            author: None,
                            published: item_published,
                            updated: None,
                        });
                        in_item = false;
                        item_title.clear();
                        item_link.clear();
                        item_desc.clear();
                        item_published = None;
                    }
                    current_tag.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(FeedError::Parse(ParserError {
                        tag: current_tag,
                        message: e.to_string(),
                    }));
                }
                _ => {}
            }
        }
        Ok(Feed {
            title,
            link,
            description: if description.is_empty() {
                None
            } else {
                Some(description)
            },
            updated: None,
            entries,
        })
    }
}

fn parse_rfc2822(text: &str) -> Result<Option<DateTime<Utc>>, ParserError> {
    match DateTime::parse_from_rfc2822(text) {
        Ok(dt) => Ok(Some(dt.into())),
        Err(_) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_rss() {
        let xml = r#"<?xml version="1.0"?>
                <rss version="2.0">
                    <channel>
                        <title>My Blog</title>
                        <link>https://example.com</link>
                        <description>A blog</description>
                        <item>
                            <title>First Post</title>
                            <link>https://example.com/1</link>
                            <description>Hello world</description>
                        </item>
                    </channel>
                </rss>"#;

        let feed = RssParser.parse(xml).unwrap();
        assert_eq!(feed.title, "My Blog");
        assert_eq!(feed.link, "https://example.com");
        assert_eq!(feed.entries.len(), 1);
        assert_eq!(feed.entries[0].title.as_deref(), Some("First Post"));
    }

    #[test]
    fn parse_rss_with_dates_multi_items() {
        let xml = r#"<?xml version="1.0"?>
                <rss version="2.0"><channel>
                    <title>Tech News</title>
                    <link>https://tech.example.com</link>
                    <description>Tech</description>
                    <item>
                        <title>Post A</title>
                        <link>https://tech.example.com/a</link>
                        <pubDate>Mon, 01 Jan 2024 12:00:00 +0000</pubDate>
                    </item>
                    <item>
                        <title>Post B</title>
                        <link>https://tech.example.com/b</link>
                        <!-- no pubDate -->
                    </item>
                </channel></rss>"#;

        let feed = RssParser.parse(xml).unwrap();
        assert_eq!(feed.entries.len(), 2);
        assert!(feed.entries[0].published.is_some());
        assert!(feed.entries[1].published.is_none());
    }

    #[test]
    fn parse_rss_cdata_content() {
        let xml = r#"<?xml version="1.0"?>
            <rss version="2.0"><channel>
                <title><![CDATA[Hacker News: Front Page]]></title>
                <link>https://news.ycombinator.com/</link>
                <description>Hacker News RSS</description>
                <item>
                    <title><![CDATA[How to Earn a Billion Dollars]]></title>
                    <link>https://paulgraham.com/earn.html</link>
                    <description><![CDATA[<p>Article URL: <a href="https://example.com">link</a></p>]]></description>
                </item>
            </channel></rss>"#;

        let feed = RssParser.parse(xml).unwrap();
        assert_eq!(feed.title, "Hacker News: Front Page");
        assert_eq!(
            feed.entries[0].title.as_deref(),
            Some("How to Earn a Billion Dollars")
        );
        assert!(
            feed.entries[0]
                .summary
                .as_ref()
                .unwrap()
                .contains("<p>Article URL:")
        );
    }

    #[test]
    fn parse_rss_general_ref() {
        let xml = r#"<?xml version="1.0"?>
                        <rss version="2.0"><channel>
                            <title><![CDATA[Hacker News: Front Page]]></title>
                            <link>https://news.ycombinator.com/</link>
                            <description>Hacker News RSS</description>
                            <item>
                                <title>Hello &amp; goodbye &lt;world&gt;</title>
                                <link>https://paulgraham.com/earn.html</link>
                                <description><![CDATA[<p>Article URL: <a href="https://example.com">link</a></p>]]></description>
                            </item>
                        </channel></rss>"#;
        let feed = RssParser.parse(xml).unwrap();
        assert_eq!(
            feed.entries[0].title.as_deref(),
            Some("Hello & goodbye <world>")
        );
    }
}
