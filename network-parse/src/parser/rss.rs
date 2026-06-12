use super::FeedParser;
use crate::{
    error::{FeedError, ParserError},
    model::{Entry, Feed},
};
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
                    let text = String::from_utf8_lossy(ev.as_ref()).trim().to_string();
                    if text.is_empty() {
                        continue;
                    }
                    if in_item {
                        match current_tag.as_str() {
                            "title" => item_title = text,
                            "link" => item_link = text,
                            "description" => item_desc = text,
                            _ => {}
                        }
                    } else if in_channel {
                        match current_tag.as_str() {
                            "title" => title = text,
                            "link" => link = text,
                            "description" => description = text,
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
                                Some(item_title.clone())
                            },
                            link: if item_link.is_empty() {
                                None
                            } else {
                                Some(item_link.clone())
                            },
                            summary: if item_desc.is_empty() {
                                None
                            } else {
                                Some(item_desc.clone())
                            },
                            content: None,
                            author: None,
                            published: None,
                            updated: None,
                        });
                        in_item = false;
                        item_title.clear();
                        item_link.clear();
                        item_desc.clear();
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
}
