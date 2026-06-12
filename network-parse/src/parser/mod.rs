use crate::{error::FeedError, model::Feed};

pub trait FeedParser {
    fn parse(&self, xml: &str) -> Result<Feed, FeedError> {
        todo!()
    }
}

pub mod atom;
pub mod rss;
