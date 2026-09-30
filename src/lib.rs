//! Fast crawler/bot detection from User-Agent strings.
//!
//! [`is_crawler`] classifies with a keyword/engine heuristic. With the `database`
//! feature, `crawler_info` returns matching Crawlerdex metadata.
//!
//! ```
//! use iscrawl::is_crawler;
//!
//! assert!(is_crawler("Googlebot/2.1 (+http://www.google.com/bot.html)"));
//! assert!(!is_crawler(
//!     "Mozilla/5.0 (X11; Linux x86_64; rv:115.0) Gecko/20100101 Firefox/115.0"
//! ));
//! ```
//!
//! # Heuristic
//!
//! 1. Empty input: crawler.
//! 2. Input over 512 bytes: browser (not classified).
//! 3. Crawler keyword (`bot`, `crawl`, `spider`, `http`, `@`, ...): crawler.
//! 4. No browser engine token (`gecko`, `webkit`, `msie`, ...): crawler.
//! 5. Otherwise: browser.

#![deny(missing_docs)]

#[cfg(feature = "database")]
mod database;

#[cfg(feature = "database")]
pub use database::{CrawlerInfo, crawler_info};

use std::cell::RefCell;

const MAX_LENGTH: usize = 512;
const CACHE_SLOTS: usize = 256;

const CRAWLER_KEYWORDS: &[&[u8]] = &[
    b"http", b"bot", b"crawl", b"spider", b"checker", b"feed", b"fetch", b"monitor",
    b"ptst", b"preview",
];

const BROWSER_ENGINES: &[&[u8]] = &[
    b"gecko",
    b"webkit",
    b"msie",
    b"trident",
    b"opera",
    b"konqueror",
    b"links",
    b"icab",
    b"netfront",
    b"lynx",
    b"mosaic",
    b"netsurf",
];

static NEEDLE_STARTS: [bool; 65536] = mark_starts(
    mark_starts([false; 65536], CRAWLER_KEYWORDS),
    BROWSER_ENGINES,
);

const fn mark_starts(mut table: [bool; 65536], needles: &[&[u8]]) -> [bool; 65536] {
    let mut index = 0;
    while index < needles.len() {
        table[pair_index(needles[index][0], needles[index][1])] = true;
        index += 1;
    }
    table
}

const fn pair_index(first: u8, second: u8) -> usize {
    (first as usize) << 8 | second as usize
}

#[derive(Clone, Copy, PartialEq)]
struct Input {
    key: u64,
    length: usize,
    head: u64,
    tail: u64,
}

thread_local! {
    static CACHE: RefCell<[Option<(Input, bool)>; CACHE_SLOTS]> =
        const { RefCell::new([None; CACHE_SLOTS]) };
}

/// Returns `true` if `user_agent` looks like a crawler/bot, `false` if it
/// looks like a human browser. See the crate docs for the heuristic.
///
/// ```
/// use iscrawl::is_crawler;
///
/// assert!(is_crawler("Mozilla/5.0 (compatible; bingbot/2.0; +http://www.bing.com)"));
/// ```
pub fn is_crawler(user_agent: &str) -> bool {
    let bytes = user_agent.as_bytes();
    let input = Input {
        key: (bytes.as_ptr() as u64).rotate_left(17) ^ bytes.len() as u64,
        length: bytes.len(),
        head: edge_word(bytes),
        tail: edge_word(&bytes[bytes.len().saturating_sub(8)..]),
    };

    CACHE.with_borrow_mut(|cache| {
        let slot = &mut cache[input.key as usize % CACHE_SLOTS];
        if let Some((_, result)) = slot.filter(|(cached, _)| *cached == input) {
            return result;
        }

        let result = classify(bytes);
        *slot = Some((input, result));
        result
    })
}

fn edge_word(bytes: &[u8]) -> u64 {
    match bytes.first_chunk() {
        Some(chunk) => u64::from_ne_bytes(*chunk),
        None => bytes
            .iter()
            .fold(0, |word, &byte| word << 8 | u64::from(byte)),
    }
}

fn classify(source: &[u8]) -> bool {
    if source.is_empty() {
        return true;
    }
    if source.len() > MAX_LENGTH {
        return false;
    }

    let mut buffer = [0; MAX_LENGTH];
    let lowered = &mut buffer[..source.len()];
    lowered.copy_from_slice(source);
    lowered.make_ascii_lowercase();

    if lowered.contains(&b'@') {
        return true;
    }

    let mut has_engine = false;
    for (position, pair) in lowered.windows(2).enumerate() {
        if !NEEDLE_STARTS[pair_index(pair[0], pair[1])] {
            continue;
        }

        let rest = &lowered[position..];
        if starts_with_any(rest, CRAWLER_KEYWORDS) {
            return true;
        }
        has_engine = has_engine || starts_with_any(rest, BROWSER_ENGINES);
    }
    !has_engine
}

fn starts_with_any(bytes: &[u8], needles: &[&[u8]]) -> bool {
    needles.iter().any(|needle| bytes.starts_with(needle))
}
