//! Simple library for converting [JSON with Comments] into [JSON],
//! in short it removes the following:
//!
//! - Line comments, e.g. `// Line Comment`
//! - Block comments, e.g. `/* Block Comment */`
//! - Trailing commas, e.g. `[1,2,3,,]` -> `[1,2,3]`
//!
//! **Note:** The implementation does not use a full-blown [JSON with Comments]
//! parser. Instead it uses a [JSON with Comments] tokenizer, which makes
//! conversion a lot faster.
//!
//! Currently `#![no_std]` is not supported. It will however be added, when
//! some upstream changes have been applied.
//!
//! See [`jsonc_to_json()`] for more information.
//!
//! # Example
//!
//! ```rust
//! use jsonc_to_json::{jsonc_to_json, jsonc_to_json_into};
//!
//! let jsonc = "{\"arr\": [1, 2,/* Comment */ 3, 4,,]}// Line Comment";
//!
//! let json = jsonc_to_json(jsonc);
//! println!("{}", json);
//! # assert_eq!(json, "{\"arr\": [1, 2, 3, 4]}");
//!
//! // Alternatively, use `jsonc_to_json_into()` to reuse an
//! // already allocated `String`
//! let mut json = String::new();
//! jsonc_to_json_into(jsonc, &mut json);
//! println!("{}", json);
//! # assert_eq!(json, "{\"arr\": [1, 2, 3, 4]}");
//! ```
//!
//! Both outputs the following:
//!
//! ```text
//! {"arr": [1, 2, 3, 4]}
//! ```
//!
//! # Non-Allocating Iterator Example
//!
//! Non-allocating [`Iterator`] that yields string slices of
//! valid [JSON].
//!
//! ```rust
//! use jsonc_to_json::jsonc_to_json_iter;
//!
//! let jsonc = r#"{foo}/**/[1,2,3,,]"bar""#;
//!
//! let mut iter = jsonc_to_json_iter(jsonc);
//! assert_eq!(iter.next(), Some("{foo}")); // Line comment was removed
//! assert_eq!(iter.next(), Some("[1,2,3")); // Trailing commas was removed
//! assert_eq!(iter.next(), Some("]\"bar\""));
//! assert_eq!(iter.next(), None);
//! ```
//!
//! [JSON with Comments]: https://code.visualstudio.com/docs/languages/json#_json-with-comments
//! [JSON]: https://www.json.org/json-en.html

#![forbid(unsafe_code)]
#![forbid(elided_lifetimes_in_paths)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![warn(clippy::all)]

use std::borrow::Cow;
use std::iter::FusedIterator;
use std::ops::Range;

use any_lexer::{JsonCLexer, JsonCToken, Lexer, TokenSpan};

/// Removes all [JSON with Comments] parts from `jsonc`, turning it into
/// valid [JSON], i.e. removing line comments, block comments, and trailing
/// commas.
///
/// - Line comments, e.g. `// Line Comment`
/// - Block comments, e.g. `/* Block Comment */`
/// - Trailing commas, e.g. `[1,2,3,,]` -> `[1,2,3]`
///
/// If `jsonc` is already valid [JSON], then <code>[Cow]::[Borrowed]\(jsonc)</code>
/// is returned, otherwise a new [`String`] is allocated and <code>[Cow]::[Owned]</code>
/// is returned.
///
/// **Warning:** The conversion is infallible and does not validate `jsonc`.
/// If it contains invalid [JSON] or invalid [JSON with Comments], then the
/// invalid parts are included in the result, i.e. `{foo,/*comment*/bar,}`
/// is turned into `{foo,bar}`.
///
/// See also [`jsonc_to_json_into()`] for an alternative variant, that reuses
/// an already allocated [`String`].
///
/// # Example
///
/// ```rust
/// use jsonc_to_json::{jsonc_to_json, jsonc_to_json_into};
///
/// let jsonc = "{\"arr\": [1, 2,/* Comment */ 3, 4,,]}// Line Comment";
///
/// let json = jsonc_to_json(jsonc);
/// println!("{}", json);
/// # assert_eq!(json, "{\"arr\": [1, 2, 3, 4]}");
/// ```
///
/// Which outputs the following:
///
/// ```text
/// {"arr": [1, 2, 3, 4]}
/// ```
///
/// [JSON with Comments]: https://code.visualstudio.com/docs/languages/json#_json-with-comments
/// [JSON]: https://www.json.org/json-en.html
/// [Borrowed]: Cow::Borrowed
/// [Owned]: Cow::Owned
/// [examples/example.rs]: https://github.com/vallentin/jsonc-to-json/blob/master/examples/example.rs
pub fn jsonc_to_json(jsonc: &str) -> Cow<'_, str> {
    let mut iter = JsonCToJsonIter::new(jsonc);

    let first = match iter.next() {
        Some(first) => first,
        None => return Cow::Borrowed(""),
    };

    let second = match iter.next() {
        Some(second) => second,
        None => return Cow::Borrowed(first),
    };

    let mut json = String::new();
    json.push_str(first);
    json.push_str(second);

    for part in iter {
        json.push_str(part);
    }

    Cow::Owned(json)
}

/// Same as [`jsonc_to_json()`], but instead of allocating a
/// new [`String`], then the output JSON is appended to `json`.
///
/// **Note:** The output [JSON] is appended to `json`, i.e. if `json`
/// is not empty, then call [`clear()`] beforehand.
///
/// See [`jsonc_to_json()`] for more information.
///
/// # Example
///
/// ```rust
/// # use jsonc_to_json::jsonc_to_json_into;
/// let jsonc = "{\"arr\": [1, 2,/* Comment */ 3, 4,,]}// Line Comment";
///
/// let mut json = String::new();
/// jsonc_to_json_into(jsonc, &mut json);
/// println!("{}", json);
/// # assert_eq!(json, "{\"arr\": [1, 2, 3, 4]}");
/// ```
///
/// Which outputs the following:
///
/// ```text
/// {"arr": [1, 2, 3, 4]}
/// ```
///
/// [JSON]: https://www.json.org/json-en.html
/// [`clear()`]: String::clear
#[inline]
pub fn jsonc_to_json_into(jsonc: &str, json: &mut String) {
    for part in JsonCToJsonIter::new(jsonc) {
        json.push_str(part);
    }
}

/// Non-allocating [`Iterator`] that yields string slices of
/// valid [JSON].
///
/// **Warning:** The conversion is infallible and does not validate `jsonc`.
/// If it contains invalid [JSON] or invalid [JSON with Comments], then the
/// invalid parts are included in the result, i.e. `{foo,/*comment*/bar,}`
/// is turned into `{foo,bar}`.
///
/// See [`jsonc_to_json()`] for more information.
///
/// # Example
///
/// ```rust
/// # use jsonc_to_json::jsonc_to_json_iter;
/// let jsonc = r#"{foo}/**/[1,2,3,,]"bar""#;
///
/// let mut iter = jsonc_to_json_iter(jsonc);
/// assert_eq!(iter.next(), Some("{foo}")); // Line comment was removed
/// assert_eq!(iter.next(), Some("[1,2,3")); // Trailing commas was removed
/// assert_eq!(iter.next(), Some("]\"bar\""));
/// assert_eq!(iter.next(), None);
/// ```
///
/// [JSON]: https://www.json.org/json-en.html
#[inline]
pub fn jsonc_to_json_iter(jsonc: &str) -> JsonCToJsonIter<'_> {
    JsonCToJsonIter::new(jsonc)
}

/// See [`jsonc_to_json_iter()`] for more information.
#[derive(Clone, Debug)]
pub struct JsonCToJsonIter<'jsonc> {
    lexer: JsonCLexer<'jsonc>,
    next: Option<Range<usize>>,
}

impl<'jsonc> JsonCToJsonIter<'jsonc> {
    /// See [`jsonc_to_json_iter()`] for more information.
    pub fn new(jsonc: &'jsonc str) -> Self {
        Self {
            lexer: JsonCLexer::new(jsonc),
            next: None,
        }
    }
}

impl<'jsonc> Iterator for JsonCToJsonIter<'jsonc> {
    type Item = &'jsonc str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut span = match self.next.take() {
            Some(span) => span,
            None => self.lexer.next_valid_json_token()?,
        };

        loop {
            let next = self.lexer.next_valid_json_token();
            if let Some(next) = next {
                match span.continue_range(&next) {
                    Some(new_span) => {
                        span = new_span;
                    }
                    None => {
                        self.next = Some(next);
                        break;
                    }
                }
            } else {
                break;
            }
        }

        Some(&self.lexer.scanner().text()[span])
    }
}

impl FusedIterator for JsonCToJsonIter<'_> {}

trait JsonCToJsonExt<'jsonc> {
    fn next_token(&mut self) -> Option<(JsonCToken, &'jsonc str)>;
    fn next_valid_json_token(&mut self) -> Option<Range<usize>>;
}

impl<'jsonc, I> JsonCToJsonExt<'jsonc> for I
where
    I: Iterator<Item = (JsonCToken, TokenSpan<'jsonc>)>,
    I: Clone,
{
    #[inline]
    fn next_token(&mut self) -> Option<(JsonCToken, &'jsonc str)> {
        let (tok, span) = self.next()?;
        Some((tok, span.as_str()))
    }

    fn next_valid_json_token(&mut self) -> Option<Range<usize>> {
        loop {
            let (tok, span) = self.next()?;
            let s = span.as_str();

            match tok {
                JsonCToken::Space => {}
                JsonCToken::LineComment | JsonCToken::BlockComment => continue,
                JsonCToken::Punct if s == "," => {
                    let mut iter = self.clone().filter(|(tok, _span)| {
                        !matches!(
                            tok,
                            JsonCToken::Space | JsonCToken::LineComment | JsonCToken::BlockComment
                        )
                    });

                    let (tok, s) = match iter.next_token() {
                        Some((tok, s)) => (tok, s),
                        None => continue,
                    };

                    match tok {
                        JsonCToken::Punct if s == "," => continue,
                        JsonCToken::Delim => continue,
                        JsonCToken::String
                        | JsonCToken::Number
                        | JsonCToken::Null
                        | JsonCToken::True
                        | JsonCToken::False
                        | JsonCToken::Punct
                        | JsonCToken::Unknown => {}
                        JsonCToken::Space | JsonCToken::LineComment | JsonCToken::BlockComment => {
                            unreachable!()
                        }
                    }
                }
                JsonCToken::String
                | JsonCToken::Number
                | JsonCToken::Null
                | JsonCToken::True
                | JsonCToken::False
                | JsonCToken::Punct
                | JsonCToken::Delim
                | JsonCToken::Unknown => {}
            }

            return Some(span.range());
        }
    }
}

trait ContinueRange: Sized {
    fn continue_range(&self, next: &Self) -> Option<Self>;
}

impl ContinueRange for Range<usize> {
    #[inline]
    fn continue_range(&self, next: &Self) -> Option<Self> {
        if self.end == next.start {
            Some(self.start..next.end)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_jsonc_to_json {
        ($jsonc:expr, $json:expr) => {{
            let jsonc: &str = $jsonc;
            let json: Cow<'_, str> = $json;
            let actual = jsonc_to_json(jsonc);
            assert_eq!(actual, json);
            assert_eq!(
                matches!(actual, Cow::Borrowed(_)),
                matches!(json, Cow::Borrowed(_))
            );
        }};
    }

    #[test]
    fn test_empty() {
        assert_jsonc_to_json!("", Cow::Borrowed(""));
    }

    #[test]
    fn test_borrowed() {
        let jsonc = r#"{"arr": [1, 2, 3, 4]}"#;
        assert_jsonc_to_json!(jsonc, Cow::Borrowed(jsonc));
    }

    #[test]
    fn test_borrowed_ending_removed() {
        let jsonc = r#"{"arr": [1, 2, 3, 4]} // Line Comment"#;
        let json = r#"{"arr": [1, 2, 3, 4]} "#;
        assert_jsonc_to_json!(jsonc, Cow::Borrowed(json));
    }

    #[test]
    fn test_line_comment() {
        let jsonc = r#"// Comment
{
    //
    "arr": [1, 2,
    // Comment
    3, 4] // Comment
    //
}
// Comment"#;
        let json = "\n{\n    \n    \"arr\": [1, 2,\n    \n    3, 4] \n    \n}\n";
        assert_jsonc_to_json!(jsonc, Cow::Owned(json.to_owned()));
    }

    #[test]
    fn test_iter() {
        let jsonc = r#"{foo}/**/[1,2,3,,]"bar""#;
        let mut iter = jsonc_to_json_iter(jsonc);

        assert_eq!(iter.next(), Some("{foo}"));
        assert_eq!(iter.next(), Some("[1,2,3"));
        assert_eq!(iter.next(), Some("]\"bar\""));
        assert_eq!(iter.next(), None);
    }
}
