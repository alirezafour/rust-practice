use chrono::{DateTime, Utc};
use quick_xml::{Reader, events::Event};

use crate::{
    error::{FeedError, ParserError},
    model::{Author, Entry, Feed},
    parser::FeedParser,
};

pub struct AtomParser;

impl FeedParser for AtomParser {
    fn parse(&self, xml: &str) -> Result<Feed, FeedError> {
        let mut reader = Reader::from_str(xml);

        let mut in_entry = false;
        let mut in_author = false;
        let mut current_tag = String::new();

        // accumulate data
        let mut title = String::new();
        let mut link = String::new();
        let mut description: Option<String> = None;
        let mut updated: Option<DateTime<Utc>> = None;
        let mut entries: Vec<Entry> = Vec::new();

        // current entry
        let mut entry_id = String::new();
        let mut entry_title = String::new();
        let mut entry_link = String::new();
        let mut entry_desc = String::new();
        let mut entry_published: Option<DateTime<Utc>> = None;
        let mut entry_updated: Option<DateTime<Utc>> = None;
        let mut entry_author: Option<Author> = None;

        // current author
        let mut author_name = String::new();

        loop {
            match reader.read_event() {
                Ok(Event::Empty(ev)) => {
                    let tag = String::from_utf8_lossy(ev.name().as_ref()).to_string();
                    if tag == "link" {
                        for attr in ev.attributes().flatten() {
                            if attr.key.as_ref() == b"href" {
                                let href = String::from_utf8_lossy(&attr.value).to_string();
                                if in_entry {
                                    entry_link = href;
                                } else {
                                    link = href;
                                }
                            }
                        }
                    }
                }
                Ok(Event::Start(ev)) => {
                    let tag = String::from_utf8_lossy(ev.name().as_ref()).to_string();
                    match tag.as_str() {
                        "entry" => in_entry = true,
                        "author" => in_author = true,
                        _ => current_tag = tag,
                    }
                }
                Ok(Event::Text(ev)) => {
                    let text = String::from_utf8_lossy(ev.as_ref()).to_string();
                    if text.is_empty() {
                        continue;
                    }
                    if in_author {
                        if current_tag.as_str() == "name" {
                            author_name = text;
                        }
                    } else if in_entry {
                        match current_tag.as_str() {
                            "id" => entry_id = text,
                            "title" => entry_title = text,
                            "link" => entry_link = text,
                            "summary" => entry_desc = text,
                            "published" => entry_published = parse_rfc3339(&text)?,
                            "updated" => entry_updated = parse_rfc3339(&text)?,
                            _ => {}
                        }
                    } else {
                        match current_tag.as_str() {
                            "title" => title = text,
                            "subtitle" => description = Some(text),
                            "updated" => updated = parse_rfc3339(&text)?,
                            _ => {}
                        }
                    }
                }
                Ok(Event::CData(ev)) => {
                    let text = String::from_utf8_lossy(ev.as_ref()).trim().to_string();
                    if text.is_empty() {
                        continue;
                    }
                    if in_author {
                        if current_tag.as_str() == "name" {
                            author_name = text;
                        }
                    } else if in_entry {
                        match current_tag.as_str() {
                            "id" => entry_id = text,
                            "title" => entry_title = text,
                            "link" => entry_link = text,
                            "summary" => entry_desc = text,
                            "published" => entry_published = parse_rfc3339(&text)?,
                            "updated" => entry_updated = parse_rfc3339(&text)?,
                            _ => {}
                        }
                    } else {
                        match current_tag.as_str() {
                            "title" => title = text,
                            "subtitle" => description = Some(text),
                            "updated" => updated = parse_rfc3339(&text)?,
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ev)) => {
                    let tag = String::from_utf8_lossy(ev.name().as_ref())
                        .trim()
                        .to_string();
                    if tag == "entry" && in_entry {
                        entries.push(Entry {
                            id: entry_id.clone(),
                            title: if entry_title.is_empty() {
                                None
                            } else {
                                Some(entry_title.clone())
                            },
                            link: if entry_link.is_empty() {
                                None
                            } else {
                                Some(entry_link.clone())
                            },
                            summary: if entry_desc.is_empty() {
                                None
                            } else {
                                Some(entry_desc.clone())
                            },
                            content: None,
                            author: entry_author,
                            published: entry_published,
                            updated: entry_updated,
                        });
                        in_entry = false;
                        entry_id.clear();
                        entry_title.clear();
                        entry_link.clear();
                        entry_desc.clear();
                        entry_published = None;
                        entry_updated = None;
                        entry_author = None;
                    }
                    if tag == "author" && in_author {
                        entry_author = Some(Author {
                            name: author_name.clone(),
                            email: None,
                            uri: None,
                        });
                        in_author = false;
                        author_name.clear();
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
            description,
            updated,
            entries,
        })
    }
}

fn parse_rfc3339(text: &str) -> Result<Option<DateTime<Utc>>, ParserError> {
    match DateTime::parse_from_rfc3339(text) {
        Ok(dt) => Ok(Some(dt.into())),
        Err(_) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_atom() {
        let xml = r#"<?xml version="1.0"?>
                <feed xmlns="http://www.w3.org/2005/Atom">
                    <title>My Blog</title>
                    <link href="https://example.com"/>
                    <subtitle>A blog</subtitle>
                    <updated>2024-01-01T12:00:00Z</updated>
                    <entry>
                        <title>First Post</title>
                        <link href="https://example.com/1"/>
                        <id>post-1</id>
                        <published>2024-01-01T10:00:00Z</published>
                        <updated>2024-01-01T12:00:00Z</updated>
                        <summary>Hello world</summary>
                        <author><name>Alice</name></author>
                    </entry>
                </feed>"#;
        let feed = AtomParser.parse(xml).unwrap();
        assert_eq!(feed.title, "My Blog");
        assert_eq!(feed.link, "https://example.com");
        assert_eq!(feed.entries.len(), 1);
        assert_eq!(feed.entries[0].title.as_deref(), Some("First Post"));
        assert_eq!(feed.entries[0].author.as_ref().unwrap().name, "Alice");
        assert!(feed.entries[0].published.is_some());
    }

    #[test]
    fn parse_atom_multi_entry_id_isolation() {
        let xml = r#"<?xml version="1.0"?>
            <feed xmlns="http://www.w3.org/2005/Atom">
                <title>Blog</title>
                <link href="https://example.com"/>
                <entry>
                    <id>post-1</id>
                    <title>First</title>
                    <link href="https://example.com/1"/>
                </entry>
                <entry>
                    <title>Second</title>
                    <link href="https://example.com/2"/>
                    <!-- no id — must NOT inherit post-1 -->
                </entry>
            </feed>"#;
        let feed = AtomParser.parse(xml).unwrap();
        assert_eq!(feed.entries.len(), 2);
        assert_eq!(feed.entries[0].id, "post-1");
        assert_eq!(feed.entries[1].id, ""); // not "post-1"
    }

    #[test]
    fn parse_atom_dates_normalize_to_utc() {
        let xml = r#"<?xml version="1.0"?>
            <feed xmlns="http://www.w3.org/2005/Atom">
                <title>Blog</title>
                <link href="https://example.com"/>
                <entry>
                    <title>Dated</title>
                    <link href="https://example.com/1"/>
                    <id>1</id>
                    <published>2024-01-01T10:00:00+02:00</published>
                </entry>
            </feed>"#;
        let feed = AtomParser.parse(xml).unwrap();
        let published = feed.entries[0].published.unwrap();
        // +02:00 offset → 08:00 UTC
        assert_eq!(published.format("%H:%M").to_string(), "08:00");
    }
}
