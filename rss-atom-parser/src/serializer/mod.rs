use crate::{error::FeedError, model::Feed};

pub trait FeedSerializer {
    fn serialize(&self, _feed: &Feed) -> Result<Vec<u8>, FeedError> {
        todo!()
    }
}

pub mod atom;
pub mod json;
pub mod rss;
