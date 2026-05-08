use iscrawl::is_crawler;
use std::fs;

const MAX_FALSE_POSITIVE_RATE: f64 = 1.0;

fn load(path: &str) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect()
}

fn false_positive_rate(lines: &[String]) -> f64 {
    let fps = lines.iter().filter(|line| is_crawler(line)).count();
    100.0 * fps as f64 / lines.len() as f64
}

#[test]
fn browser_corpus_low_false_positives() {
    let lines = load("tests/fixtures/browser_user_agents.txt");
    let rate = false_positive_rate(&lines);
    assert!(
        rate <= MAX_FALSE_POSITIVE_RATE,
        "browser FP {rate:.4}% > {MAX_FALSE_POSITIVE_RATE}%"
    );
}

#[test]
fn browser_corpus_non_empty() {
    let lines = load("tests/fixtures/browser_user_agents.txt");
    assert!(!lines.is_empty());
}

#[test]
fn browser_corpus_substantial_size() {
    let lines = load("tests/fixtures/browser_user_agents.txt");
    assert!(
        lines.len() >= 1000,
        "browser corpus too small: {}",
        lines.len()
    );
}

#[test]
fn typical_chrome_uas_pass() {
    let lines = load("tests/fixtures/browser_user_agents.txt");
    let chrome_lines: Vec<&String> = lines
        .iter()
        .filter(|line| line.contains("Chrome/") && !line.contains("bot"))
        .take(50)
        .collect();
    let misclassified = chrome_lines.iter().filter(|line| is_crawler(line)).count();
    let total = chrome_lines.len();
    assert!(
        misclassified * 100 <= total,
        "Chrome FP rate too high: {misclassified}/{total}"
    );
}

#[test]
fn typical_firefox_uas_pass() {
    let lines = load("tests/fixtures/browser_user_agents.txt");
    let firefox_lines: Vec<&String> = lines
        .iter()
        .filter(|line| line.contains("Firefox/"))
        .take(50)
        .collect();
    if firefox_lines.is_empty() {
        return;
    }
    for line in firefox_lines {
        assert!(!is_crawler(line), "Firefox FP: {line}");
    }
}

#[test]
fn typical_safari_uas_pass() {
    let lines = load("tests/fixtures/browser_user_agents.txt");
    let safari_lines: Vec<&String> = lines
        .iter()
        .filter(|line| line.contains("Safari/") && !line.contains("Chrome"))
        .take(50)
        .collect();
    if safari_lines.is_empty() {
        return;
    }
    let misclassified = safari_lines.iter().filter(|line| is_crawler(line)).count();
    let total = safari_lines.len();
    assert!(
        misclassified * 100 <= total,
        "Safari FP rate too high: {misclassified}/{total}"
    );
}

#[test]
fn no_panic_across_full_browser_corpus() {
    let lines = load("tests/fixtures/browser_user_agents.txt");
    for line in &lines {
        let _ = is_crawler(line);
    }
}

#[test]
fn browser_corpus_classification_stable() {
    let lines = load("tests/fixtures/browser_user_agents.txt");
    for line in lines.iter().take(500) {
        let first = is_crawler(line);
        let second = is_crawler(line);
        assert_eq!(first, second, "unstable result for: {line}");
    }
}
