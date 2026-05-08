use iscrawl::is_crawler;

#[test]
fn mozilla_with_engine_and_compatible_browser() {
    assert!(!is_crawler(
        "Mozilla/5.0 (compatible; X11; Linux) Gecko/20100101 Firefox/115.0"
    ));
}

#[test]
fn mozilla_with_engine_no_compatible_not_crawler() {
    assert!(!is_crawler(
        "Mozilla/5.0 (X11; Linux) Gecko/20100101 CustomAgent/1.0"
    ));
}

#[test]
fn mozilla_no_engine_with_compatible_is_crawler() {
    assert!(is_crawler("Mozilla/5.0 (compatible; SomeAgent/1.0)"));
}

#[test]
fn mozilla_no_engine_no_compatible_not_crawler() {
    assert!(!is_crawler("Mozilla/5.0 something/1.0"));
}

#[test]
fn opera_prefix_with_engine_and_compatible_browser() {
    assert!(!is_crawler(
        "Opera/9.80 (compatible; Windows NT 6.1) Presto/2.12.388 Version/12.18"
    ));
}

#[test]
fn opera_prefix_no_compatible_with_engine_not_crawler() {
    assert!(!is_crawler(
        "Opera/9.80 (Windows NT 6.1) Presto/2.12.388 Version/12.18"
    ));
}

#[test]
fn non_mozilla_with_engine_not_crawler() {
    assert!(!is_crawler("CustomAgent Gecko/1.0"));
}

#[test]
fn non_mozilla_no_engine_is_crawler() {
    assert!(is_crawler("CustomAgent/1.0"));
}

#[test]
fn compatible_token_partial_does_not_count() {
    assert!(!is_crawler(
        "Mozilla/5.0 (compatibility; X11; Linux) Gecko/20100101 Firefox/115.0"
    ));
}

#[test]
fn compatible_with_lowercase_required() {
    assert!(!is_crawler(
        "Mozilla/5.0 (COMPATIBLE; X11) Gecko/20100101 Firefox/115.0"
    ));
}

#[test]
fn mozilla_lowercase_prefix_works() {
    assert!(!is_crawler(
        "mozilla/5.0 (compatible; X11; Linux) Gecko/20100101 Firefox/115.0"
    ));
}

#[test]
fn mozilla_uppercase_prefix_works() {
    assert!(!is_crawler(
        "MOZILLA/5.0 (compatible; X11; Linux) Gecko/20100101 Firefox/115.0"
    ));
}

#[test]
fn mozilla_followed_by_keyword_overrides_browser_logic() {
    assert!(is_crawler(
        "Mozilla/5.0 (compatible; MyBot/1.0) Gecko/20100101 Firefox/115.0"
    ));
}

#[test]
fn mozilla_with_http_marker_detected() {
    assert!(is_crawler(
        "Mozilla/5.0 (compatible; Acme/1.0; +http://acme.com/about)"
    ));
}

#[test]
fn mozilla_with_email_marker_detected() {
    assert!(is_crawler(
        "Mozilla/5.0 (compatible; Acme/1.0; ops@acme.com)"
    ));
}

#[test]
fn opera_with_keyword_overrides() {
    assert!(is_crawler("Opera/9.80 spider/1.0"));
}

#[test]
fn just_mozilla_prefix_no_paren_not_crawler() {
    assert!(!is_crawler("Mozilla/5.0"));
}

#[test]
fn just_compatible_token_no_mozilla_not_crawler() {
    assert!(!is_crawler("CustomAgent (compatible; foo) Gecko/1.0"));
}

#[test]
fn compatible_appears_only_with_mozilla_check() {
    assert!(!is_crawler("Mozilla/5.0 compatible; foo) Gecko/1.0"));
}

#[test]
fn opera_with_compatible_and_engine_is_browser() {
    assert!(!is_crawler(
        "Opera/12.0 (compatible; Linux) Presto/2.12 Version/12.0"
    ));
}
