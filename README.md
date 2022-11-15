# HTTP accept header

A very basic & naive implementation of the HTTP Accept header. It uses [http](https://crates.io/crates/http) crate and [mime](https://crates.io/crates/mime) crate to parse the accept header. Basic data structure:

```rust
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
```

Usage:

```rust
// parse accept header
let accept: Accept = "application/json, text/html;q=0.9, text/plain;q=0.8, */*;q=0.7"
    .parse()
    .unwrap();

// prepare a list of supported media types
let available = vec![
    Mime::from_str("text/html").unwrap(),
    Mime::from_str("application/json").unwrap(),
];

// content negotiation
let negotiated = accept.negotiate(&available).unwrap();

// "application/json" shall be chosen since it is available and has the highest weight
assert_eq!(negotiated, Mime::from_str("application/json").unwrap());
```
