use iscrawl::is_crawler;
use std::fs;

const MIN_DETECTION_RATE: f64 = 94.0;

fn load(path: &str) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect()
}

fn detection_rate(lines: &[String]) -> f64 {
    let hits = lines.iter().filter(|line| is_crawler(line)).count();
    100.0 * hits as f64 / lines.len() as f64
}

#[test]
fn crawler_user_agents_high_detection() {
    let lines = load("tests/fixtures/crawler_user_agents.txt");
    let rate = detection_rate(&lines);
    assert!(
        rate >= MIN_DETECTION_RATE,
        "crawler_user_agents.txt detection {rate:.2}% < {MIN_DETECTION_RATE}%"
    );
}

#[test]
fn loadkpi_crawlers_high_detection() {
    let lines = load("tests/fixtures/loadkpi_crawlers.txt");
    let rate = detection_rate(&lines);
    assert!(
        rate >= MIN_DETECTION_RATE,
        "loadkpi_crawlers.txt detection {rate:.2}% < {MIN_DETECTION_RATE}%"
    );
}

#[test]
fn pgts_crawlers_high_detection() {
    let lines = load("tests/fixtures/crawler_user_agents_pgts.txt");
    let rate = detection_rate(&lines);
    assert!(
        rate >= MIN_DETECTION_RATE,
        "crawler_user_agents_pgts.txt detection {rate:.2}% < {MIN_DETECTION_RATE}%"
    );
}

#[test]
fn combined_crawler_corpus_high_detection() {
    let mut lines = load("tests/fixtures/crawler_user_agents.txt");
    lines.extend(load("tests/fixtures/loadkpi_crawlers.txt"));
    lines.extend(load("tests/fixtures/crawler_user_agents_pgts.txt"));
    let rate = detection_rate(&lines);
    assert!(
        rate >= MIN_DETECTION_RATE,
        "combined corpus detection {rate:.2}% < {MIN_DETECTION_RATE}%"
    );
}

#[test]
fn crawler_corpus_non_empty() {
    let lines = load("tests/fixtures/crawler_user_agents.txt");
    assert!(!lines.is_empty());
}

#[test]
fn loadkpi_corpus_non_empty() {
    let lines = load("tests/fixtures/loadkpi_crawlers.txt");
    assert!(!lines.is_empty());
}

#[test]
fn pgts_corpus_non_empty() {
    let lines = load("tests/fixtures/crawler_user_agents_pgts.txt");
    assert!(!lines.is_empty());
}

#[test]
fn googlebot_present_in_corpus_detected() {
    let lines = load("tests/fixtures/crawler_user_agents.txt");
    let google_lines: Vec<&String> = lines
        .iter()
        .filter(|line| line.to_lowercase().contains("googlebot"))
        .collect();
    assert!(!google_lines.is_empty(), "no Googlebot in fixture");
    for line in google_lines {
        assert!(is_crawler(line), "Googlebot UA missed: {line}");
    }
}

#[test]
fn bingbot_present_in_corpus_detected() {
    let lines = load("tests/fixtures/crawler_user_agents.txt");
    let bing_lines: Vec<&String> = lines
        .iter()
        .filter(|line| line.to_lowercase().contains("bingbot"))
        .collect();
    if bing_lines.is_empty() {
        return;
    }
    for line in bing_lines {
        assert!(is_crawler(line), "bingbot UA missed: {line}");
    }
}

#[test]
fn each_corpus_line_under_buffer_or_handled() {
    let mut lines = load("tests/fixtures/crawler_user_agents.txt");
    lines.extend(load("tests/fixtures/loadkpi_crawlers.txt"));
    lines.extend(load("tests/fixtures/crawler_user_agents_pgts.txt"));
    for line in &lines {
        let _ = is_crawler(line);
    }
}
