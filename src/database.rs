use aho_corasick::{AhoCorasick, AhoCorasickKind};
use serde::Deserialize;
use std::sync::LazyLock;

const CRAWLER_DATABASE: &str = include_str!("../crawlers.min.json");

/// uaid metadata for a matched User-Agent.
#[derive(Debug, Deserialize)]
pub struct CrawlerInfo {
    /// Lowercase substrings tested against the lowercased User-Agent.
    #[serde(rename = "match")]
    pub rules: Vec<Rule>,
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

/// One uaid match rule: a substring, or several that must all appear.
/// A leading `^` anchors a substring to the start of the User-Agent.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Rule {
    /// A single substring.
    Part(String),
    /// Substrings that must all appear.
    Parts(Vec<String>),
}

struct Candidate {
    crawler: usize,
    rest: Vec<&'static str>,
}

struct Matchers {
    automaton: AhoCorasick,
    candidates: Vec<Candidate>,
}

static CRAWLERS: LazyLock<Vec<CrawlerInfo>> = LazyLock::new(|| {
    serde_json::from_str(CRAWLER_DATABASE).expect("bundled crawler database is valid")
});

static MATCHERS: LazyLock<Matchers> = LazyLock::new(|| {
    let mut rules: Vec<(usize, Vec<&'static str>)> = CRAWLERS
        .iter()
        .enumerate()
        .flat_map(|(crawler, info)| {
            info.rules.iter().map(move |rule| (crawler, parts(rule)))
        })
        .collect();
    rules.sort_by_key(|(_, parts)| {
        std::cmp::Reverse(parts.iter().map(|part| part.len()).sum::<usize>())
    });

    let needles: Vec<&str> = rules.iter().map(|(_, parts)| parts[0]).collect();
    let candidates = rules
        .into_iter()
        .map(|(crawler, parts)| Candidate {
            crawler,
            rest: parts[1..].to_vec(),
        })
        .collect();

    Matchers {
        automaton: AhoCorasick::builder()
            .kind(Some(AhoCorasickKind::DFA))
            .build(&needles)
            .expect("bundled needles are valid"),
        candidates,
    }
});

fn parts(rule: &'static Rule) -> Vec<&'static str> {
    let mut parts: Vec<&str> = match rule {
        Rule::Part(part) => vec![part],
        Rule::Parts(parts) => parts.iter().map(String::as_str).collect(),
    };
    parts.sort_by_key(|part| std::cmp::Reverse(part.len()));
    parts
}

/// Returns uaid metadata for `user_agent`, or `None` when no rule matches.
///
/// The database is bundled from [uaid](https://github.com/tn3w/uaid):
///
/// ```bash
/// curl -fsSL https://raw.githubusercontent.com/tn3w/uaid/main/crawlers.json \
///   | jq -c 'map(del(.instances, .addition_date, .depends_on))' > crawlers.min.json
/// ```
///
/// Matching is case-insensitive. One Aho-Corasick automaton finds the longest part of
/// every rule; the remaining parts are checked only for those hits. The most specific
/// (longest) matching rule wins.
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
    let lowered = format!("^{}", user_agent.to_lowercase());

    let best = matchers
        .automaton
        .find_overlapping_iter(&lowered)
        .map(|found| found.pattern().as_usize())
        .filter(|&index| {
            let rest = &matchers.candidates[index].rest;
            rest.iter().all(|part| lowered.contains(part))
        })
        .min()?;

    Some(&CRAWLERS[matchers.candidates[best].crawler])
}
