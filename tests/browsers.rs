use iscrawl::is_crawler;

#[test]
fn firefox_modern_not_crawler() {
    assert!(!is_crawler(
        "Mozilla/5.0 (X11; Linux x86_64; rv:115.0) Gecko/20100101 Firefox/115.0"
    ));
}

#[test]
fn chrome_windows_not_crawler() {
    assert!(!is_crawler(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
         Chrome/120.0.0.0 Safari/537.36"
    ));
}

#[test]
fn safari_macos_not_crawler() {
    assert!(!is_crawler(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 \
         (KHTML, like Gecko) Version/17.0 Safari/605.1.15"
    ));
}

#[test]
fn edge_windows_not_crawler() {
    assert!(!is_crawler(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
         Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0"
    ));
}

#[test]
fn opera_not_crawler() {
    assert!(!is_crawler(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
         Chrome/120.0.0.0 Safari/537.36 OPR/106.0.0.0"
    ));
}

#[test]
fn old_msie_not_crawler() {
    assert!(!is_crawler(
        "Mozilla/4.0 (compatible; MSIE 8.0; Windows NT 6.1; Trident/4.0)"
    ));
}

#[test]
fn presto_opera_not_crawler() {
    assert!(!is_crawler(
        "Opera/9.80 (Windows NT 6.1) Presto/2.12.388 Version/12.18"
    ));
}

#[test]
fn konqueror_not_crawler() {
    assert!(!is_crawler(
        "Mozilla/5.0 (compatible; Konqueror/4.14; Linux) KHTML/4.14.16 (like Gecko)"
    ));
}

#[test]
fn icab_not_crawler() {
    assert!(!is_crawler(
        "Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10.6; en-US) iCab/4.8 (like Gecko)"
    ));
}

#[test]
fn links_not_crawler() {
    assert!(!is_crawler(
        "Links (2.27; Linux 6.5.0 x86_64; GNU C 13.2; text)"
    ));
}

#[test]
fn android_chrome_not_crawler() {
    assert!(!is_crawler(
        "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) \
         Chrome/120.0.0.0 Mobile Safari/537.36"
    ));
}

#[test]
fn iphone_safari_not_crawler() {
    assert!(!is_crawler(
        "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 \
         (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1"
    ));
}

#[test]
fn random_alpha_string_classified_crawler() {
    assert!(is_crawler("randomstring"));
}

#[test]
fn no_engine_no_mozilla_classified_crawler() {
    assert!(is_crawler("python-requests/2.28.1"));
}

#[test]
fn wget_classified_crawler() {
    assert!(is_crawler("Wget/1.21.3"));
}

#[test]
fn java_client_classified_crawler() {
    assert!(is_crawler("Java/17.0.5"));
}

#[test]
fn go_http_client_classified_crawler() {
    assert!(is_crawler("Go-http-client/1.1"));
}

#[test]
fn mozilla_prefix_no_compatible_no_engine_not_crawler() {
    assert!(!is_crawler("Mozilla/5.0 something custom"));
}

#[test]
fn mozilla_with_engine_no_compatible_not_crawler() {
    assert!(!is_crawler(
        "Mozilla/5.0 AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0"
    ));
}

#[test]
fn case_insensitive_engine_match() {
    assert!(!is_crawler(
        "Mozilla/5.0 (X11; Linux) GECKO/20100101 FIREFOX/115.0"
    ));
}
