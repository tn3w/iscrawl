use aho_corasick::{AhoCorasick, AhoCorasickKind};
use regex::Regex;
use regex_syntax::hir::literal::{ExtractKind, Extractor};
use serde::Deserialize;
use std::sync::LazyLock;

const CRAWLER_DATABASE: &str = include_str!("../crawlers.min.json");
const MIN_LITERAL_LENGTH: usize = 3;

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
    automaton: AhoCorasick,
    candidates: Vec<Candidate>,
    ungated: Vec<(usize, Regex)>,
}

struct Candidate {
    crawler: usize,
    verify: Option<Regex>,
}

static CRAWLERS: LazyLock<Vec<CrawlerInfo>> = LazyLock::new(|| {
    serde_json::from_str(CRAWLER_DATABASE).expect("bundled crawler database is valid")
});

static MATCHERS: LazyLock<Matchers> = LazyLock::new(|| {
    let mut needles = Vec::new();
    let mut candidates = Vec::new();
    let mut ungated = Vec::new();

    for (crawler, info) in CRAWLERS.iter().enumerate() {
        if let Some(literal) = as_literal(&info.pattern) {
            needles.push(literal.into_bytes());
            candidates.push(Candidate {
                crawler,
                verify: None,
            });
            continue;
        }

        let Ok(regex) = Regex::new(&info.pattern) else {
            continue;
        };
        let Some(gates) = required_literals(&info.pattern) else {
            ungated.push((crawler, regex));
            continue;
        };
        for gate in gates {
            needles.push(gate);
            candidates.push(Candidate {
                crawler,
                verify: Some(regex.clone()),
            });
        }
    }

    Matchers {
        automaton: AhoCorasick::builder()
            .kind(Some(AhoCorasickKind::DFA))
            .build(&needles)
            .expect("bundled needles are valid"),
        candidates,
        ungated,
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

fn required_literals(pattern: &str) -> Option<Vec<Vec<u8>>> {
    let hir = regex_syntax::parse(pattern).ok()?;

    [ExtractKind::Prefix, ExtractKind::Suffix]
        .into_iter()
        .filter_map(|kind| {
            let sequence = Extractor::new().kind(kind).extract(&hir);
            Some(
                sequence
                    .literals()?
                    .iter()
                    .map(|literal| literal.as_bytes().to_vec())
                    .collect(),
            )
        })
        .filter(|literals: &Vec<Vec<u8>>| shortest_length(literals) >= MIN_LITERAL_LENGTH)
        .max_by_key(|literals| shortest_length(literals))
}

fn shortest_length(literals: &[Vec<u8>]) -> usize {
    literals.iter().map(Vec::len).min().unwrap_or(0)
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
/// Matching uses one Aho-Corasick automaton over literals and required regex prefixes;
/// candidate regexes run only when their prefix appears.
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
    let is_verified = |candidate: &&Candidate| {
        let verify = candidate.verify.as_ref();
        verify.is_none_or(|regex| regex.is_match(user_agent))
    };

    let gated = matchers
        .automaton
        .find_overlapping_iter(user_agent)
        .map(|found| &matchers.candidates[found.pattern()])
        .find(is_verified)
        .map(|candidate| candidate.crawler);
    let crawler = gated.or_else(|| {
        let (crawler, _) = matchers
            .ungated
            .iter()
            .find(|(_, regex)| regex.is_match(user_agent))?;
        Some(*crawler)
    })?;

    Some(&CRAWLERS[crawler])
}
