use crate::{error::*, Accept, MediaType};
use http::StatusCode;
use mime::Mime;
use std::{cmp::Ordering, str::FromStr};

impl Accept {
    /// Determine the most suitable `Content-Type` encoding.
    pub fn negotiate(&self, available: &[Mime]) -> Result<Mime, StatusCode> {
        for media_type in &self.types {
            if available.contains(&media_type.mime) {
                return Ok(media_type.mime.clone());
            }
        }

        if self.wildcard.is_some() {
            if let Some(accept) = available.iter().next() {
                return Ok(accept.clone());
            }
        }

        Err(StatusCode::NOT_ACCEPTABLE)
    }
}

impl FromStr for Accept {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut types = Vec::new();
        let mut wildcard = None;

        for part in s.split(',') {
            let mtype: MediaType = part.trim().parse()?;

            if mtype.mime.type_() == mime::STAR {
                wildcard = Some(mtype);
            } else {
                types.push(mtype);
            }
        }

        // if there is a wildcard weight, apply it to all other types which doesn't has weight
        if let Some(w) = wildcard.as_ref() {
            for mtype in &mut types {
                if mtype.weight.is_none() {
                    mtype.weight = w.weight;
                }
            }
        }

        types.sort_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        Ok(Accept { wildcard, types })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_should_be_parsed_and_sorted() {
        let accept = "application/json, text/html;q=0.9, text/plain;q=0.8, */*;q=0.7, */*;q=0.6"
            .parse::<Accept>()
            .unwrap();

        assert_eq!(
            accept.wildcard,
            Some(MediaType::from_str("*/*; q=0.6").unwrap())
        );
        assert_eq!(accept.types.len(), 3);
        assert_eq!(accept.types[0].mime, Mime::from_str("text/html").unwrap());
        assert_eq!(accept.types[0].weight, Some(0.9));
        assert_eq!(accept.types[1].mime, Mime::from_str("text/plain").unwrap());
        assert_eq!(accept.types[1].weight, Some(0.8));
        assert_eq!(
            accept.types[2].mime,
            Mime::from_str("application/json").unwrap()
        );
        assert_eq!(accept.types[2].weight, Some(0.6));
    }

    #[test]
    fn content_negotiation_should_work() {
        let accept = "application/json, text/html;q=0.9, text/plain;q=0.8, */*;q=0.7"
            .parse::<Accept>()
            .unwrap();

        let available = vec![
            Mime::from_str("text/html").unwrap(),
            Mime::from_str("application/json").unwrap(),
        ];

        let negotiated = accept.negotiate(&available).unwrap();

        assert_eq!(negotiated, Mime::from_str("text/html").unwrap());

        let available = vec![Mime::from_str("application/xml").unwrap()];
        let negotiated = accept.negotiate(&available).unwrap();
        assert_eq!(negotiated, Mime::from_str("application/xml").unwrap());
    }

    #[test]
    fn content_negotiation_should_fail_if_no_available() {
        let accept = "application/json,text/html;q=0.9,text/plain;q=0.8"
            .parse::<Accept>()
            .unwrap();

        let available = vec![Mime::from_str("application/xml").unwrap()];
        let negotiated = accept.negotiate(&available);
        assert_eq!(negotiated, Err(StatusCode::NOT_ACCEPTABLE));
    }
}