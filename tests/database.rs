#![cfg(feature = "database")]

use iscrawl::crawler_info;

#[test]
fn googlebot_info_returned() {
    let info = crawler_info("Googlebot/2.1 (+http://www.google.com/bot.html)").unwrap();

    assert_eq!(info.pattern, "Googlebot\\/");
    assert_eq!(
        info.description,
        "Google's main web crawling bot for search indexing"
    );
    assert!(info.tags.iter().any(|tag| tag == "search-engine"));
}

#[test]
fn browser_info_none() {
    let user_agent = "Mozilla/5.0 (X11; Linux x86_64; rv:115.0) Gecko/20100101 Firefox/115.0";

    assert!(crawler_info(user_agent).is_none());
}

#[test]
fn db_only_match_is_crawler() {
    let user_agent = "YandexSomething/1.0";
    let info = crawler_info(user_agent).unwrap();

    assert_eq!(info.description, "Yandex unspecified bot");
}
