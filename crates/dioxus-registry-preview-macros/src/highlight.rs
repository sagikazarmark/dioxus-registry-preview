//! Compile-time syntax highlighting for Example sources.
//!
//! The spans use the same byte offsets and short theme tags that `dioxus-code` produces, so the
//! facade can hand them to `dioxus_code::advanced::HighlightedSource` unchanged.

use std::cell::RefCell;
use std::collections::HashMap;

use arborium::Highlighter;
use proc_macro2::TokenStream;
use quote::quote;

thread_local! {
    static HIGHLIGHTER: RefCell<Highlighter> = RefCell::new(Highlighter::new());
}

/// One rendered highlight range with its theme tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Span {
    start: u32,
    end: u32,
    tag: &'static str,
}

struct Candidate {
    start: u32,
    end: u32,
    tag: Option<&'static str>,
    pattern_index: u32,
}

/// Highlights Rust `source` and returns a `&'static [HighlightSpan]` expression.
pub(crate) fn rust_spans(docs_crate: &TokenStream, source: &str) -> Result<TokenStream, String> {
    let raw = HIGHLIGHTER
        .with(|highlighter| highlighter.borrow_mut().highlight_spans("rust", source))
        .map_err(|error| error.to_string())?;
    let spans = normalize(raw.into_iter().map(|span| Candidate {
        start: span.start,
        end: span.end,
        tag: arborium_theme::tag_for_capture(&span.capture),
        pattern_index: span.pattern_index,
    }));
    let spans = spans.iter().map(|span| {
        let Span { start, end, tag } = span;
        quote! { #docs_crate::code::HighlightSpan::new(#start..#end, #tag) }
    });

    Ok(quote! { &[#(#spans),*] })
}

// Mirrors the normalization `dioxus-code` applies to its own compile-time spans: one candidate per
// byte range, untagged captures dropped, and touching runs of one tag merged.
fn normalize(candidates: impl IntoIterator<Item = Candidate>) -> Vec<Span> {
    let mut by_range: HashMap<(u32, u32), Candidate> = HashMap::new();

    for candidate in candidates {
        let key = (candidate.start, candidate.end);
        match by_range.get(&key) {
            Some(existing) if !replaces(&candidate, existing) => {}
            _ => {
                by_range.insert(key, candidate);
            }
        }
    }

    let mut spans: Vec<Span> = by_range
        .into_values()
        .filter_map(|candidate| {
            Some(Span {
                start: candidate.start,
                end: candidate.end,
                tag: candidate.tag?,
            })
        })
        .collect();
    spans.sort_by_key(|span| (span.start, span.end));

    let mut coalesced: Vec<Span> = Vec::with_capacity(spans.len());
    for span in spans {
        if let Some(last) = coalesced.last_mut()
            && last.tag == span.tag
            && span.start <= last.end
        {
            last.end = last.end.max(span.end);
            continue;
        }
        coalesced.push(span);
    }

    coalesced
}

fn replaces(candidate: &Candidate, existing: &Candidate) -> bool {
    match (candidate.tag.is_some(), existing.tag.is_some()) {
        (true, false) => true,
        (false, true) => false,
        _ => candidate.pattern_index >= existing.pattern_index,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(start: u32, end: u32, tag: Option<&'static str>, pattern_index: u32) -> Candidate {
        Candidate {
            start,
            end,
            tag,
            pattern_index,
        }
    }

    #[test]
    fn tagged_candidates_win_over_untagged_ones_for_the_same_range() {
        let spans = normalize([
            candidate(0, 2, Some("k"), 0),
            candidate(0, 2, None, 9),
            candidate(3, 7, None, 1),
        ]);

        assert_eq!(
            spans,
            vec![Span {
                start: 0,
                end: 2,
                tag: "k",
            }]
        );
    }

    #[test]
    fn later_patterns_win_over_earlier_ones_for_the_same_range() {
        let spans = normalize([candidate(0, 2, Some("k"), 1), candidate(0, 2, Some("f"), 4)]);

        assert_eq!(spans[0].tag, "f");
    }

    #[test]
    fn touching_runs_with_one_tag_are_merged() {
        let spans = normalize([
            candidate(4, 6, Some("s"), 0),
            candidate(0, 2, Some("s"), 0),
            candidate(2, 4, Some("s"), 0),
            candidate(6, 8, Some("k"), 0),
        ]);

        assert_eq!(
            spans,
            vec![
                Span {
                    start: 0,
                    end: 6,
                    tag: "s",
                },
                Span {
                    start: 6,
                    end: 8,
                    tag: "k",
                },
            ]
        );
    }

    #[test]
    fn rust_keywords_are_highlighted() {
        let docs_crate = quote!(::dioxus_registry_preview);
        let tokens = rust_spans(&docs_crate, "fn main() {}").unwrap().to_string();

        assert!(
            tokens.contains("HighlightSpan :: new (0u32 .. 2u32 , \"k\")"),
            "{tokens}"
        );
    }
}
