use iscrawl::is_crawler;

#[test]
fn cached_classification_is_stable_for_same_input() {
    let user_agent = "Googlebot/2.1 (+http://www.google.com/bot.html)";

    assert!(is_crawler(user_agent));
    assert!(is_crawler(user_agent));
}

#[test]
fn same_length_inputs_with_different_edges_do_not_reuse_cached_result() {
    let browser = "Mozilla/5.0 Firefox/115.0";
    let crawler = "Googlebot/2.1 (+http://x)";

    assert_eq!(browser.len(), crawler.len());
    assert!(!is_crawler(browser));
    assert!(is_crawler(crawler));
    assert!(!is_crawler(browser));
}

#[test]
fn owned_strings_with_reused_capacity_classify_by_current_contents() {
    let mut user_agent = String::with_capacity(64);

    user_agent.push_str("Mozilla/5.0 Firefox/115.0");
    assert!(!is_crawler(&user_agent));

    user_agent.clear();
    user_agent.push_str("Googlebot/2.1 (+http://x)");
    assert!(is_crawler(&user_agent));

    user_agent.clear();
    user_agent.push_str("Mozilla/5.0 Firefox/115.0");
    assert!(!is_crawler(&user_agent));
}

#[test]
fn cache_churn_across_more_than_slot_count_stays_correct() {
    for i in 0..512 {
        let browser = format!("Mozilla/5.0 Firefox/{i}.0");
        let crawler = format!("CrawlerBot/{i}.0 (+http://example.com)");

        assert!(!is_crawler(&browser));
        assert!(is_crawler(&crawler));
    }
}
