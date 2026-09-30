use iscrawl::is_crawler;

const CRAWLERS: &[&str] = &[
    "",
    " ",
    "x",
    "1234567890",
    "!@#$%^&*()",
    "\x01\x02\x03",
    "ünïcödé",
    "python-requests/2.28.1",
    "curl/7.81.0",
    "Wget/1.21.3",
    "Java/17.0.5",
    "Go-http-client/1.1",
    "Mozilla/5.0",
    "Mozilla/5.0 something custom",
    "FooBar http://example.com",
    "FooBar https://example.com",
    "contact@example.com agent",
    "MyCrAwLeR/1.0",
    "YandexSpider/3.0",
    "LinkChecker/9.4",
    "FeedFetcher-Google",
    "UptimeMonitor/2.1",
    "Mozilla/5.0 PTST/1.0",
    "LinkPreview/1.0",
    "foo\tbot\tbar",
    "   crawler   ",
    "Opera/9.80 spider/1.0",
    "Mozilla/5.0 (compatible; MyBot/1.0) Gecko/20100101 Firefox/115.0",
    "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
    "Mozilla/5.0 (compatible; bingbot/2.0; +http://www.bing.com/bingbot.htm)",
    "Mozilla/5.0 (compatible; Baiduspider/2.0; +http://www.baidu.com/search/spider.html)",
    "facebookexternalhit/1.1 (+http://www.facebook.com/externalhit_uatext.php)",
    "Slackbot-LinkExpanding 1.0 (+https://api.slack.com/robots)",
    "TelegramBot (like TwitterBot)",
];

const BROWSERS: &[&str] = &[
    "Mozilla/5.0 (X11; Linux x86_64; rv:115.0) Gecko/20100101 Firefox/115.0",
    "Mozilla/5.0 (X11; Linux) GECKO/20100101 FIREFOX/115.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
     Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
     Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 \
     (KHTML, like Gecko) Version/17.0 Safari/605.1.15",
    "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) \
     Chrome/120.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 \
     (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
    "Mozilla/4.0 (compatible; MSIE 8.0; Windows NT 6.1; Trident/4.0)",
    "Opera/9.80 (Windows NT 6.1) Presto/2.12.388 Version/12.18",
    "Mozilla/5.0 (compatible; Konqueror/4.14; Linux) KHTML/4.14.16 (like Gecko)",
    "Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10.6; en-US) iCab/4.8 (like Gecko)",
    "Links (2.27; Linux 6.5.0 x86_64; GNU C 13.2; text)",
    "Lynx/2.8.6rel.4 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/1.6.3",
    "Mozilla/4.0 (compatible; Linux 2.6.22) NetFront/3.4 Kindle/2.5 (screen 600x800)",
];

#[test]
fn crawlers_detected() {
    for user_agent in CRAWLERS {
        assert!(is_crawler(user_agent), "missed crawler: {user_agent}");
    }
}

#[test]
fn browsers_accepted() {
    for user_agent in BROWSERS {
        assert!(!is_crawler(user_agent), "false positive: {user_agent}");
    }
}

#[test]
fn length_limit_is_512_bytes() {
    assert!(is_crawler(&"a".repeat(512)));
    assert!(!is_crawler(&"a".repeat(513)));
    assert!(!is_crawler(&format!("crawler {}", "x".repeat(600))));
}

#[test]
fn keyword_at_length_limit_detected() {
    assert!(is_crawler(&format!("{}bot", "x".repeat(509))));
}

#[test]
fn repeated_calls_are_stable() {
    for user_agent in CRAWLERS.iter().chain(BROWSERS) {
        assert_eq!(is_crawler(user_agent), is_crawler(user_agent));
    }
}

#[test]
fn same_length_inputs_do_not_share_cached_result() {
    let browser = "Mozilla/5.0 Gecko/115.0.1";
    let crawler = "Googlebot/2.1 (+http://x)";

    assert_eq!(browser.len(), crawler.len());
    assert!(!is_crawler(browser));
    assert!(is_crawler(crawler));
    assert!(!is_crawler(browser));
}

#[test]
fn reused_string_buffer_classifies_current_contents() {
    let mut user_agent = String::with_capacity(64);

    for text in ["Mozilla/5.0 Gecko/115.0.1", "Googlebot/2.1 (+http://x)"] {
        user_agent.clear();
        user_agent.push_str(text);
        assert_eq!(is_crawler(&user_agent), text.starts_with("Googlebot"));
    }
}

#[test]
fn cache_churn_stays_correct() {
    for version in 0..512 {
        assert!(!is_crawler(&format!("Mozilla/5.0 Gecko/{version}.0")));
        assert!(is_crawler(&format!(
            "CrawlerBot/{version}.0 (+http://example.com)"
        )));
    }
}

const KEYWORDS: &[&str] = &[
    "http", "@", "bot", "crawl", "spider", "checker", "feed", "fetch", "monitor", "ptst",
    "preview",
];

const ENGINES: &[&str] = &[
    "gecko",
    "webkit",
    "msie",
    "trident",
    "opera",
    "konqueror",
    "links",
    "icab",
    "netfront",
    "lynx",
    "mosaic",
    "netsurf",
];

#[test]
fn every_engine_token_marks_browser() {
    for engine in ENGINES {
        assert!(
            !is_crawler(&format!("Mozilla/5.0 {engine}/1.0")),
            "{engine}"
        );
        assert!(
            !is_crawler(&format!("Agent {}/1.0", engine.to_uppercase())),
            "{engine}"
        );
    }
}

#[test]
fn every_keyword_marks_crawler_at_any_position() {
    for keyword in KEYWORDS {
        for template in ["{}/1.0 Gecko/1", "Gecko/1 {}/1.0", "Gecko/1 {}", "{}"] {
            let user_agent = template.replace("{}", keyword);
            assert!(is_crawler(&user_agent), "{user_agent}");
            assert!(is_crawler(&user_agent.to_uppercase()), "{user_agent}");
        }
    }
}

#[test]
fn keyword_beats_engine() {
    for keyword in KEYWORDS {
        for engine in ENGINES {
            assert!(
                is_crawler(&format!("{engine} {keyword}")),
                "{engine} {keyword}"
            );
        }
    }
}

#[test]
fn engine_alone_at_string_boundaries_is_browser() {
    for engine in ENGINES {
        assert!(!is_crawler(engine), "{engine}");
        assert!(!is_crawler(&format!("{engine} x")), "{engine}");
        assert!(!is_crawler(&format!("x {engine}")), "{engine}");
    }
}

#[test]
fn truncated_engine_tokens_are_not_engines() {
    for engine in ENGINES {
        assert!(is_crawler(&engine[..engine.len() - 1]), "{engine}");
    }
}

#[test]
fn truncated_keywords_are_not_keywords() {
    for keyword in KEYWORDS.iter().filter(|keyword| keyword.len() > 1) {
        let truncated = format!("Gecko {}", &keyword[..keyword.len() - 1]);
        assert!(!is_crawler(&truncated), "{truncated}");
    }
}

#[test]
fn keyword_inside_larger_word_still_matches() {
    assert!(is_crawler("Gecko Googlebot"));
    assert!(is_crawler("Gecko Robotics"));
    assert!(is_crawler("Gecko Baiduspider"));
    assert!(is_crawler("Gecko Pingdom.com_bot_version_1.4"));
}

#[test]
fn engine_token_case_is_ignored() {
    for user_agent in ["GECKO", "Gecko", "gEcKo", "WebKit", "MSIE 6.0"] {
        assert!(!is_crawler(user_agent), "{user_agent}");
    }
}

#[test]
fn multibyte_characters_around_tokens() {
    assert!(is_crawler("日本語 bot 日本語"));
    assert!(is_crawler("Gecko 日本語 bot"));
    assert!(is_crawler("Gecko 日本語 user@example.com"));
    assert!(!is_crawler("日本語 Gecko 日本語"));
    assert!(!is_crawler("Gecko ünïcödé"));
}

#[test]
fn single_byte_keyword_at_last_position() {
    assert!(is_crawler("Gecko @"));
    assert!(is_crawler("Gecko user@"));
    assert!(is_crawler("@"));
}

#[test]
fn control_bytes_and_whitespace_do_not_hide_tokens() {
    assert!(is_crawler("Gecko\r\nbot\r\n"));
    assert!(is_crawler("\0bot\0"));
    assert!(!is_crawler("\tGecko\t"));
}

#[test]
fn realistic_browsers_with_tokens_in_unusual_places() {
    assert!(!is_crawler(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/117.0"
    ));
    assert!(!is_crawler(
        "Mozilla/5.0 (Windows NT 6.1; WOW64; Trident/7.0; rv:11.0) like Gecko"
    ));
    assert!(!is_crawler(
        "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) \
         SamsungBrowser/23.0 Chrome/115.0.0.0 Mobile Safari/537.36"
    ));
}

#[test]
fn disguised_browser_with_bot_suffix_detected() {
    assert!(is_crawler(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
         (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 (compatible; ExampleBot/1.0)"
    ));
}

#[test]
fn results_are_consistent_across_threads() {
    let threads: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(|| {
                for _ in 0..100 {
                    assert!(CRAWLERS.iter().all(|user_agent| is_crawler(user_agent)));
                    assert!(BROWSERS.iter().all(|user_agent| !is_crawler(user_agent)));
                }
            })
        })
        .collect();

    for thread in threads {
        thread.join().unwrap();
    }
}

#[test]
fn same_content_in_different_allocations_agrees() {
    for user_agent in CRAWLERS.iter().chain(BROWSERS) {
        let copies: Vec<String> = (0..4).map(|_| user_agent.to_string()).collect();
        assert!(
            copies
                .iter()
                .all(|copy| is_crawler(copy) == is_crawler(user_agent))
        );
    }
}

#[test]
fn substrings_of_one_buffer_classify_independently() {
    let buffer = "Gecko/1 GoogleBot Gecko/2";

    assert!(is_crawler(buffer));
    assert!(!is_crawler(&buffer[..7]));
    assert!(is_crawler(&buffer[8..17]));
    assert!(!is_crawler(&buffer[18..]));
}
