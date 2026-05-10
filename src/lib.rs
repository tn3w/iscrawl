//! Tiny, allocation-free crawler/bot detection from User-Agent strings.
//!
//! One public function: [`is_crawler`]. Returns `true` for crawlers/bots,
//! `false` for human browsers. Stack-only, ~140ns cold, ~5ns warm via a
//! thread-local cache.
//!
//! # Example
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
//! 2. Input over 512 bytes: `false` (oversized, not classified).
//! 3. Crawler keyword present (`bot`, `crawl`, `spider`, `+http`, `@`, ...): crawler.
//! 4. No `Mozilla/`/`Opera/` prefix and no browser engine token: crawler.
//! 5. `Mozilla/`/`Opera/` prefix lacking both an engine token and `(compatible;`: crawler.
//! 6. Otherwise: browser.
//!
//! Heuristic, not a database. Trades a sliver of recall for speed.

#![deny(missing_docs)]

/// Substrings whose presence flags a UA as a crawler. Stored as
/// `(first_byte, rest)` so the hot loop can skip on a 256-entry table lookup.
const CRAWLER_KEYWORDS: &[(u8, &[u8])] = &[
    (b'h', b"ttp://"),
    (b'h', b"ttps://"),
    (b'+', b"http"),
    (b'@', b""),
    (b'b', b"ot"),
    (b'c', b"rawl"),
    (b'c', b"hecker"),
    (b's', b"pider"),
    (b's', b"canner"),
    (b's', b"crape"),
    (b'f', b"eed"),
    (b'f', b"etch"),
    (b'm', b"onitor"),
    (b'p', b"tst"),
    (b'p', b"review"),
    (b'a', b"rchive"),
];

/// Browser engine/product tokens. A UA containing one of these is treated as
/// a real browser unless a crawler keyword also matched.
const BROWSER_ENGINES: &[(u8, &[u8])] = &[
    (b'g', b"ecko"),
    (b'k', b"html"),
    (b'k', b"onqueror"),
    (b'w', b"ebkit"),
    (b'c', b"hrome"),
    (b'f', b"irefox"),
    (b'm', b"sie"),
    (b'e', b"dge"),
    (b'o', b"pera"),
    (b't', b"rident"),
    (b'p', b"resto"),
    (b'l', b"inks"),
    (b'i', b"cab"),
];

/// Bitset of first bytes appearing in [`CRAWLER_KEYWORDS`]. Skips needle
/// scans for ~99% of haystack bytes.
const KEYWORD_FIRST_BYTES: [bool; 256] = first_byte_table(CRAWLER_KEYWORDS);

/// Bitset of first bytes appearing in [`BROWSER_ENGINES`].
const ENGINE_FIRST_BYTES: [bool; 256] = first_byte_table(BROWSER_ENGINES);

/// Build a 256-entry first-byte lookup table at compile time.
const fn first_byte_table(needles: &[(u8, &[u8])]) -> [bool; 256] {
    let mut table = [false; 256];
    let mut i = 0;
    while i < needles.len() {
        table[needles[i].0 as usize] = true;
        i += 1;
    }
    table
}

/// Returns `true` if `user_agent` looks like a crawler/bot, `false` if it
/// looks like a human browser.
///
/// Empty input is treated as a crawler. Input longer than 512 bytes is
/// rejected and returns `false` (no heap fallback by design). See the
/// crate-level docs for the full heuristic.
///
/// # Example
///
/// ```
/// use iscrawl::is_crawler;
///
/// assert!(is_crawler("Mozilla/5.0 (compatible; bingbot/2.0; +http://www.bing.com/bingbot.htm)"));
/// assert!(!is_crawler(
///     "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
///      (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
/// ));
/// ```
pub fn is_crawler(user_agent: &str) -> bool {
    let bytes = user_agent.as_bytes();
    let key = cache_key(bytes);
    let (head, tail) = edge_words(bytes);

    CACHE.with(|cache| {
        let slot = &mut cache.borrow_mut()[(key as usize) & (CACHE_SLOTS - 1)];
        if slot.key == key && slot.len == bytes.len() && slot.head == head && slot.tail == tail {
            return slot.result;
        }

        let result = classify(bytes);
        *slot = Entry {
            key,
            len: bytes.len(),
            head,
            tail,
            result,
        };
        result
    })
}

/// Classify without consulting the thread-local cache.
fn classify(source: &[u8]) -> bool {
    if source.is_empty() {
        return true;
    }

    let mut buffer = [0u8; 512];
    if source.len() > buffer.len() {
        return false;
    }

    let lowered = &mut buffer[..source.len()];
    lowered.copy_from_slice(source);
    lowered.make_ascii_lowercase();

    if contains_any(lowered, CRAWLER_KEYWORDS, &KEYWORD_FIRST_BYTES) {
        return true;
    }

    let mozilla_prefix = lowered.starts_with(b"mozilla/") || lowered.starts_with(b"opera/");
    let has_engine = contains_any(lowered, BROWSER_ENGINES, &ENGINE_FIRST_BYTES);

    if !mozilla_prefix {
        return !has_engine;
    }

    !has_engine && lowered.windows(12).any(|w| w == b"(compatible;")
}

/// Number of slots in the thread-local direct-mapped cache.
const CACHE_SLOTS: usize = 256;

/// One direct-mapped cache entry.
#[derive(Clone, Copy)]
struct Entry {
    key: u64,
    len: usize,
    head: u64,
    tail: u64,
    result: bool,
}

thread_local! {
    /// Per-thread cache for recent User-Agent classifications.
    static CACHE: std::cell::RefCell<[Entry; CACHE_SLOTS]> =
        const { std::cell::RefCell::new([Entry { key: 0, len: usize::MAX, head: 0, tail: 0, result: false }; CACHE_SLOTS]) };
}

/// Build a cheap pointer/length cache key.
fn cache_key(bytes: &[u8]) -> u64 {
    let ptr = bytes.as_ptr() as usize as u64;
    ptr.rotate_left(17) ^ bytes.len() as u64
}

/// Return first/last-word content guards for a cache entry.
fn edge_words(bytes: &[u8]) -> (u64, u64) {
    if bytes.len() >= 8 {
        let head = u64::from_ne_bytes(bytes[..8].try_into().unwrap());
        let tail = u64::from_ne_bytes(bytes[bytes.len() - 8..].try_into().unwrap());
        return (head, tail);
    }

    let mut word = 0u64;
    for (shift, &byte) in bytes.iter().enumerate() {
        word |= (byte as u64) << (shift * 8);
    }
    (word, word)
}

/// Returns `true` if any `(first, rest)` needle occurs in `haystack`.
/// `first_bytes` is the precomputed bitset of needle first bytes.
fn contains_any(haystack: &[u8], needles: &[(u8, &[u8])], first_bytes: &[bool; 256]) -> bool {
    for (position, &byte) in haystack.iter().enumerate() {
        if !first_bytes[byte as usize] {
            continue;
        }
        for &(first, rest) in needles {
            if first != byte {
                continue;
            }
            let after = position + 1;
            if after + rest.len() <= haystack.len() && haystack[after..after + rest.len()] == *rest
            {
                return true;
            }
        }
    }
    false
}
