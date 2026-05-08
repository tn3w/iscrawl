use iscrawl::is_crawler;

#[test]
fn http_scheme_detected() {
    assert!(is_crawler("FooBar http://example.com"));
}

#[test]
fn https_scheme_detected() {
    assert!(is_crawler("FooBar https://example.com"));
}

#[test]
fn plus_http_marker_detected() {
    assert!(is_crawler("Bot/1.0 (+http://example.com)"));
}

#[test]
fn at_sign_marker_detected() {
    assert!(is_crawler("contact@example.com agent"));
}

#[test]
fn bot_keyword_detected() {
    assert!(is_crawler("Googlebot/2.1"));
}

#[test]
fn crawl_keyword_detected() {
    assert!(is_crawler("MyCrawler/1.0"));
}

#[test]
fn checker_keyword_detected() {
    assert!(is_crawler("LinkChecker/9.4"));
}

#[test]
fn spider_keyword_detected() {
    assert!(is_crawler("YandexSpider/3.0"));
}

#[test]
fn scanner_keyword_detected() {
    assert!(is_crawler("SecurityScanner/1.2"));
}

#[test]
fn scrape_keyword_detected() {
    assert!(is_crawler("ScrapeBot/1.0"));
}

#[test]
fn feed_keyword_detected() {
    assert!(is_crawler("FeedFetcher-Google"));
}

#[test]
fn fetch_keyword_detected() {
    assert!(is_crawler("MyFetcher/1.0"));
}

#[test]
fn monitor_keyword_detected() {
    assert!(is_crawler("UptimeMonitor/2.1"));
}

#[test]
fn ptst_keyword_detected() {
    assert!(is_crawler("Mozilla/5.0 PTST/1.0"));
}

#[test]
fn preview_keyword_detected() {
    assert!(is_crawler("LinkPreview/1.0"));
}

#[test]
fn archive_keyword_detected() {
    assert!(is_crawler("archive.org_bot"));
}

#[test]
fn googlebot_detected() {
    assert!(is_crawler(
        "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)"
    ));
}

#[test]
fn bingbot_detected() {
    assert!(is_crawler(
        "Mozilla/5.0 (compatible; bingbot/2.0; +http://www.bing.com/bingbot.htm)"
    ));
}

#[test]
fn yandexbot_detected() {
    assert!(is_crawler(
        "Mozilla/5.0 (compatible; YandexBot/3.0; +http://yandex.com/bots)"
    ));
}

#[test]
fn duckduckbot_detected() {
    assert!(is_crawler(
        "DuckDuckBot/1.1; (+http://duckduckgo.com/duckduckbot.html)"
    ));
}

#[test]
fn baiduspider_detected() {
    assert!(is_crawler(
        "Mozilla/5.0 (compatible; Baiduspider/2.0; +http://www.baidu.com/search/spider.html)"
    ));
}

#[test]
fn facebookbot_detected() {
    assert!(is_crawler(
        "facebookexternalhit/1.1 (+http://www.facebook.com/externalhit_uatext.php)"
    ));
}

#[test]
fn slackbot_detected() {
    assert!(is_crawler(
        "Slackbot-LinkExpanding 1.0 (+https://api.slack.com/robots)"
    ));
}

#[test]
fn applebot_detected() {
    assert!(is_crawler(
        "Mozilla/5.0 (compatible; Applebot/0.1; +http://www.apple.com/go/applebot)"
    ));
}

#[test]
fn ahrefsbot_detected() {
    assert!(is_crawler(
        "Mozilla/5.0 (compatible; AhrefsBot/7.0; +http://ahrefs.com/robot/)"
    ));
}

#[test]
fn semrushbot_detected() {
    assert!(is_crawler(
        "Mozilla/5.0 (compatible; SemrushBot/7~bl; +http://www.semrush.com/bot.html)"
    ));
}

#[test]
fn telegram_preview_bot_detected() {
    assert!(is_crawler("TelegramBot (like TwitterBot)"));
}

#[test]
fn curl_at_sign_detected() {
    assert!(is_crawler("curl/7.81.0"));
}
