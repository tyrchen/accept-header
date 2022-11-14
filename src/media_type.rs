use mime::Mime;
use snafu::{ensure, ResultExt};

use crate::{error::*, MediaType};
use std::{cmp::Ordering, fmt, str::FromStr};

impl FromStr for MediaType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(';');
        let v = parts.next().unwrap().trim();
        let mime = v.parse().context(MediaTypeSnafu { value: v })?;
        let Some(v) = parts
            .next()
            .map(|s| s.trim())
            .and_then(|s| s.strip_prefix("q=")) else {
            return Ok(MediaType { mime, weight: None });
            };

        let weight: f32 = v.trim().parse().context(ParseWeightSnafu { value: v })?;
        ensure!(
            (0.0..=1.0).contains(&weight),
            WeightRangeSnafu { value: weight }
        );

        Ok(MediaType {
            mime,
            weight: Some(weight),
        })
    }
}

impl From<Mime> for MediaType {
    fn from(mime: Mime) -> Self {
        Self { mime, weight: None }
    }
}

impl From<MediaType> for Mime {
    fn from(media_type: MediaType) -> Self {
        media_type.mime
    }
}

impl PartialEq<Mime> for MediaType {
    fn eq(&self, other: &Mime) -> bool {
        self.mime == *other
    }
}

impl PartialEq<Mime> for &MediaType {
    fn eq(&self, other: &Mime) -> bool {
        self.mime == *other
    }
}

impl PartialOrd for MediaType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self.weight, other.weight) {
            (Some(left), Some(right)) => left.partial_cmp(&right),
            (Some(_), None) => Some(Ordering::Greater),
            (None, Some(_)) => Some(Ordering::Less),
            (None, None) => Some(Ordering::Equal),
        }
    }
}

impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.mime)?;

        if let Some(weight) = self.weight {
            write!(f, ";q={}", weight)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_media_type_should_be_parsed() {
        let t1: MediaType = "text/html; q= 0.5 ".parse().unwrap();
        let t2: MediaType = "application/json".parse().unwrap();
        let t3 = "*/*; q=0.7".parse::<MediaType>().unwrap();
        assert_eq!(t1.mime, Mime::from_str("text/html").unwrap());
        assert_eq!(t1.mime.type_(), mime::TEXT);
        assert_eq!(t1.mime.subtype(), mime::HTML);
        assert_eq!(t2.mime.type_(), mime::APPLICATION);
        assert_eq!(t2.mime.subtype(), mime::JSON);
        assert_eq!(t1.weight, Some(0.5));
        assert_eq!(t2.weight, None);
        assert_eq!(t3.mime.type_(), mime::STAR);
    }

    #[test]
    fn invalid_media_type_should_be_rejected() {
        let t1 = "text/html; q=-0.5".parse::<MediaType>();
        let t2 = "text/html; q=1.5".parse::<MediaType>();
        let t3 = "text/html; q=abcd".parse::<MediaType>();

        assert!(t1.is_err());
        assert_eq!(
            t2.unwrap_err().to_string(),
            "Weight should be 0.0-1.0. Got 1.5"
        );
        assert_eq!(t3.unwrap_err().to_string(), "Invalid weight: abcd");
    }

    #[test]
    fn media_type_should_be_comparable() {
        let t1: MediaType = "text/html; q= 0.5 ".parse().unwrap();
        let t2: MediaType = "application/json".parse().unwrap();
        let t3: MediaType = "text/html".parse().unwrap();
        assert!(t1 > t2);
        assert!(t1 > t3);
        assert!(t2 < t1);
        assert!(t2 != t3);
        assert!(t3 < t1);
    }

    #[test]
    fn media_type_to_string_should_work() {
        let t1: MediaType = "text/html; q= 0.5 ".parse().unwrap();
        let t2: MediaType = "application/json".parse().unwrap();
        let t3: MediaType = "text/html".parse().unwrap();
        assert_eq!(t1.to_string(), "text/html;q=0.5");
        assert_eq!(t2.to_string(), "application/json");
        assert_eq!(t3.to_string(), "text/html");
    }
}
