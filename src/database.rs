use aho_corasick::AhoCorasick;
use regex::{Regex, RegexSet};
use serde::Deserialize;
use std::sync::LazyLock;

const CRAWLER_DATABASE: &str = include_str!("../crawlers.min.json");
const REGEXES_PER_CHUNK: usize = 128;

/// Crawlerdex metadata for a matched User-Agent pattern.
#[derive(Debug, Deserialize)]
pub struct CrawlerInfo {
    /// Regex pattern matched against the User-Agent.
    pub pattern: String,
    /// Upstream documentation URL, when known.
    #[serde(default)]
    pub url: Option<String>,
    /// Human-readable crawler description.
    pub description: String,
    /// Crawler category tags.
    pub tags: Vec<String>,
    /// Reverse-DNS suffixes, when known.
    #[serde(default)]
    pub rdns: Vec<String>,
}

struct Matchers {
    literals: AhoCorasick,
    literal_crawlers: Vec<usize>,
    regex_chunks: Vec<RegexChunk>,
}

struct RegexChunk {
    set: RegexSet,
    crawlers: Vec<usize>,
}

static CRAWLERS: LazyLock<Vec<CrawlerInfo>> = LazyLock::new(|| {
    serde_json::from_str(CRAWLER_DATABASE).expect("bundled crawler database is valid")
});

static MATCHERS: LazyLock<Matchers> = LazyLock::new(|| {
    let mut literals = Vec::new();
    let mut literal_crawlers = Vec::new();
    let mut regexes = Vec::new();

    for (index, crawler) in CRAWLERS.iter().enumerate() {
        if let Some(literal) = as_literal(&crawler.pattern) {
            literals.push(literal);
            literal_crawlers.push(index);
        } else if Regex::new(&crawler.pattern).is_ok() {
            regexes.push((index, crawler.pattern.as_str()));
        }
    }

    Matchers {
        literals: AhoCorasick::new(&literals).expect("bundled literals are valid"),
        literal_crawlers,
        regex_chunks: regexes
            .chunks(REGEXES_PER_CHUNK)
            .map(compile_chunk)
            .collect(),
    }
});

fn as_literal(pattern: &str) -> Option<String> {
    let mut literal = String::with_capacity(pattern.len());
    let mut chars = pattern.chars();

    while let Some(char) = chars.next() {
        match char {
            '\\' => literal.push(literal_escape(chars.next()?)?),
            _ if is_regex_meta(char) => return None,
            _ => literal.push(char),
        }
    }

    (!literal.is_empty()).then_some(literal)
}

fn literal_escape(char: char) -> Option<char> {
    "/.-_ :)(!".contains(char).then_some(char)
}

fn is_regex_meta(char: char) -> bool {
    ".^$*+?{}[]|()".contains(char)
}

fn compile_chunk(entries: &[(usize, &str)]) -> RegexChunk {
    RegexChunk {
        set: RegexSet::new(entries.iter().map(|(_, pattern)| pattern))
            .expect("validated regexes are valid"),
        crawlers: entries.iter().map(|(index, _)| *index).collect(),
    }
}

/// Returns Crawlerdex metadata for `user_agent`, or `None` when no database
/// pattern matches.
///
/// The database is bundled from Crawlerdex:
///
/// ```bash
/// curl -fsSL \
///   https://github.com/tn3w/Crawlerdex/releases/latest/download/crawlers.min.json \
///   -o crawlers.min.json
/// ```
///
/// Matching uses Aho-Corasick for literal patterns, then chunked `RegexSet`s.
///
/// # Example
///
/// ```
/// use iscrawl::crawler_info;
///
/// let info = crawler_info("Googlebot/2.1").unwrap();
/// assert!(info.tags.iter().any(|tag| tag == "search-engine"));
/// ```
pub fn crawler_info(user_agent: &str) -> Option<&'static CrawlerInfo> {
    let matchers = &*MATCHERS;

    if let Some(found) = matchers.literals.find(user_agent) {
        return Some(&CRAWLERS[matchers.literal_crawlers[found.pattern()]]);
    }

    matchers.regex_chunks.iter().find_map(|chunk| {
        let first = chunk.set.matches(user_agent).iter().next()?;
        Some(&CRAWLERS[chunk.crawlers[first]])
    })
}
