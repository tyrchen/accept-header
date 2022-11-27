use crate::{error::*, Accept, MediaType};
use http::StatusCode;
use itertools::Itertools;
use mime::Mime;
use std::{cmp::Ordering, collections::HashSet, fmt, str::FromStr};

impl Accept {
    /// Determine the most suitable `Content-Type` encoding.
    pub fn negotiate(&self, available: &HashSet<Mime>) -> Result<Mime, StatusCode> {
        for media_type in &self.types {
            if available.contains(&media_type.mime) {
                return Ok(media_type.mime.clone());
            }
        }

        if self.wildcard.is_some() {
            if available.contains(&mime::TEXT_HTML) {
                return Ok(mime::TEXT_HTML);
            }

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

        types.sort_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        Ok(Accept { wildcard, types })
    }
}

impl From<Mime> for Accept {
    fn from(mime: Mime) -> Self {
        Self {
            wildcard: None,
            types: vec![MediaType::from(mime)],
        }
    }
}

impl fmt::Display for Accept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.types.iter().map(|m| m.to_string()).join(", "))?;

        if let Some(wildcard) = &self.wildcard {
            write!(f, ", {}", wildcard)?;
        }

        Ok(())
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
        assert_eq!(
            accept.types[0].mime,
            Mime::from_str("application/json").unwrap()
        );
        assert_eq!(accept.types[0].weight, None);
        assert_eq!(accept.types[1].mime, Mime::from_str("text/html").unwrap());
        assert_eq!(accept.types[1].weight, Some(0.9));
        assert_eq!(accept.types[2].mime, Mime::from_str("text/plain").unwrap());
        assert_eq!(accept.types[2].weight, Some(0.8));
    }

    #[test]
    fn content_negotiation_should_work() {
        let accept = "application/json, text/html;q=0.9, text/plain;q=0.8, */*;q=0.7"
            .parse::<Accept>()
            .unwrap();

        let available = vec![
            Mime::from_str("text/html").unwrap(),
            Mime::from_str("application/json").unwrap(),
        ]
        .into_iter()
        .collect();

        let negotiated = accept.negotiate(&available).unwrap();

        assert_eq!(negotiated, Mime::from_str("application/json").unwrap());

        let available = vec![Mime::from_str("application/xml").unwrap()]
            .into_iter()
            .collect();
        let negotiated = accept.negotiate(&available).unwrap();
        assert_eq!(negotiated, Mime::from_str("application/xml").unwrap());
    }

    #[test]
    fn content_negotiation_should_fail_if_no_available() {
        let accept = "application/json,text/html;q=0.9,text/plain;q=0.8"
            .parse::<Accept>()
            .unwrap();

        let available = vec![Mime::from_str("application/xml").unwrap()]
            .into_iter()
            .collect();
        let negotiated = accept.negotiate(&available);
        assert_eq!(negotiated, Err(StatusCode::NOT_ACCEPTABLE));
    }

    #[test]
    fn accept_to_string_should_work() {
        let accept = "application/json, text/plain;q=0.8, text/html;q=0.9, */*;q=0.7,*/*;q=0.6"
            .parse::<Accept>()
            .unwrap();

        assert_eq!(
            accept.to_string(),
            "application/json, text/html;q=0.9, text/plain;q=0.8, */*;q=0.6"
        );
    }
}
