mod accept;
mod media_type;

pub mod error;

use mime::Mime;

#[derive(Debug, Clone, PartialEq)]
pub struct Accept {
    pub wildcard: Option<MediaType>,
    pub types: Vec<MediaType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MediaType {
    pub mime: Mime,
    pub weight: Option<f32>,
}
