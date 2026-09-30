#![cfg(feature = "database")]

use iscrawl::crawler_info;
use std::fs;

const KNOWN_CRAWLERS: &[(&str, &str, &str)] = &[
    (
        "Googlebot/2.1 (+http://www.google.com/bot.html)",
        "Googlebot\\/",
        "search-engine",
    ),
    (
        "Mozilla/5.0 (compatible; bingbot/2.0)",
        "bingbot",
        "search-engine",
    ),
    (
        "Mozilla/5.0 (compatible; YandexBot/3.0)",
        "YandexBot",
        "search-engine",
    ),
    ("DuckDuckBot/1.1", "DuckDuckBot", "search-engine"),
    ("Baiduspider/2.0", "Baiduspider", "search-engine"),
    ("Slackbot-LinkExpanding 1.0", "Slackbot", "social-preview"),
    ("facebookexternalhit/1.1", "facebook", "social-preview"),
    ("curl/7.81.0", "^curl", "http-library"),
    (
        "Mozilla/5.0 (compatible; AhrefsBot/7.0; +http://ahrefs.com/robot/)",
        "Ahrefs(Bot|SiteAudit)",
        "seo",
    ),
    (
        "Mozilla/5.0 (compatible; SemrushBot/7~bl)",
        "S[eE][mM]rushBot",
        "seo",
    ),
];

const BROWSERS: &[&str] = &[
    "Mozilla/5.0 (X11; Linux x86_64; rv:115.0) Gecko/20100101 Firefox/115.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
     Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 \
     (KHTML, like Gecko) Version/17.0 Safari/605.1.15",
];

fn load(file: &str) -> Vec<String> {
    let text = fs::read_to_string(format!("tests/fixtures/{file}")).unwrap();
    text.lines().map(String::from).collect()
}

#[test]
fn googlebot_info_has_all_fields() {
    let info = crawler_info("Googlebot/2.1 (+http://www.google.com/bot.html)").unwrap();

    assert_eq!(info.pattern, "Googlebot\\/");
    assert_eq!(info.url.as_deref(), Some("http://www.google.com/bot.html"));
    assert_eq!(
        info.description,
        "Google's main web crawling bot for search indexing"
    );
    assert_eq!(info.tags, ["search-engine"]);
    assert!(info.rdns.contains(&".googlebot.com".to_string()));
}

#[test]
fn known_crawlers_match_expected_pattern_and_tag() {
    for (user_agent, pattern, tag) in KNOWN_CRAWLERS {
        let info =
            crawler_info(user_agent).unwrap_or_else(|| panic!("no info: {user_agent}"));

        assert_eq!(info.pattern, *pattern, "{user_agent}");
        assert!(
            info.tags.iter().any(|candidate| candidate == tag),
            "{user_agent}"
        );
    }
}

#[test]
fn optional_fields_default_to_empty() {
    let info = crawler_info("curl/7.81.0").unwrap();

    assert!(info.url.is_some());
    assert!(info.rdns.is_empty());
}

#[test]
fn browsers_have_no_info() {
    for user_agent in BROWSERS {
        assert!(crawler_info(user_agent).is_none(), "{user_agent}");
    }
}

#[test]
fn empty_and_unknown_inputs_have_no_info() {
    for user_agent in ["", " ", "x", "unknown-agent-without-a-database-entry"] {
        assert!(crawler_info(user_agent).is_none(), "{user_agent:?}");
    }
}

#[test]
fn matching_is_case_sensitive() {
    assert!(crawler_info("Googlebot/2.1").is_some());
    assert!(crawler_info("googlebot/2.1").is_none());
}

#[test]
fn anchored_pattern_only_matches_at_start() {
    assert_eq!(crawler_info("curl/7.81.0").unwrap().pattern, "^curl");
    assert_ne!(
        crawler_info("Mozilla/5.0 curl/7.81.0").map(|info| info.pattern.as_str()),
        Some("^curl")
    );
}

#[test]
fn repeated_lookups_return_the_same_static_record() {
    let first = crawler_info("Googlebot/2.1").unwrap();
    let second = crawler_info("Googlebot/2.1").unwrap();

    assert!(std::ptr::eq(first, second));
}

#[test]
fn crawler_corpus_mostly_has_info() {
    let user_agents = load("crawler_user_agents.txt");
    let known = user_agents
        .iter()
        .filter(|line| crawler_info(line).is_some())
        .count();

    assert!(
        known * 100 >= user_agents.len() * 99,
        "{known}/{}",
        user_agents.len()
    );
}

#[test]
fn browser_corpus_rarely_has_info() {
    let user_agents = load("browser_user_agents.txt");
    let known = user_agents
        .iter()
        .filter(|line| crawler_info(line).is_some())
        .count();

    assert!(
        known * 500 <= user_agents.len(),
        "{known}/{}",
        user_agents.len()
    );
}

#[test]
fn every_returned_record_is_complete() {
    for user_agent in load("crawler_user_agents.txt") {
        let Some(info) = crawler_info(&user_agent) else {
            continue;
        };

        assert!(!info.pattern.is_empty(), "{user_agent}");
        assert!(!info.description.is_empty(), "{user_agent}");
        assert!(!info.tags.is_empty(), "{user_agent}");
    }
}

#[test]
fn concurrent_first_use_is_consistent() {
    let threads: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(|| crawler_info("Googlebot/2.1").unwrap().pattern.clone())
        })
        .collect();

    for thread in threads {
        assert_eq!(thread.join().unwrap(), "Googlebot\\/");
    }
}

#[test]
fn info_is_independent_of_heuristic() {
    assert!(iscrawl::is_crawler("curl/7.81.0"));
    assert!(crawler_info("curl/7.81.0").is_some());
    assert!(iscrawl::is_crawler("some-unknown-tool/1.0"));
    assert!(crawler_info("some-unknown-tool/1.0").is_none());
}

#[test]
fn regex_patterns_verified_after_literal_gate() {
    let semrush = crawler_info("SemrushBot/7").unwrap();
    assert_eq!(semrush.pattern, "S[eE][mM]rushBot");
    assert!(crawler_info("SEMRUSHBOT/7").is_none());
    assert!(crawler_info("Ahrefs").is_none());
    assert!(crawler_info("PixCCriteo").is_none());
    assert!(crawler_info("Pix and Criteo").is_some());
}
