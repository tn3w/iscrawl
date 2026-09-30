use iscrawl::is_crawler;
use std::fs;

const CRAWLER_FILES: [&str; 3] = [
    "crawler_user_agents.txt",
    "loadkpi_crawlers.txt",
    "crawler_user_agents_pgts.txt",
];
const MIN_DETECTION_PERCENT: f64 = 95.0;
const MAX_FALSE_POSITIVE_PERCENT: f64 = 0.5;

fn load(file: &str) -> Vec<String> {
    let text = fs::read_to_string(format!("tests/fixtures/{file}")).unwrap();
    text.lines()
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect()
}

fn crawler_percent(user_agents: &[String]) -> f64 {
    let hits = user_agents.iter().filter(|line| is_crawler(line)).count();
    100.0 * hits as f64 / user_agents.len() as f64
}

#[test]
fn crawler_corpora_detected() {
    for file in CRAWLER_FILES {
        let percent = crawler_percent(&load(file));
        assert!(percent >= MIN_DETECTION_PERCENT, "{file}: {percent:.2}%");
    }
}

#[test]
fn browser_corpus_has_few_false_positives() {
    let user_agents = load("browser_user_agents.txt");
    let percent = crawler_percent(&user_agents);

    assert!(user_agents.len() >= 1000);
    assert!(percent <= MAX_FALSE_POSITIVE_PERCENT, "{percent:.3}%");
}
