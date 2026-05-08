use iscrawl::is_crawler;

#[test]
fn empty_string_is_crawler() {
    assert!(is_crawler(""));
}

#[test]
fn single_space_classified_crawler() {
    assert!(is_crawler(" "));
}

#[test]
fn single_letter_classified_crawler() {
    assert!(is_crawler("x"));
}

#[test]
fn random_short_token_classified_crawler() {
    assert!(is_crawler("abc"));
}

#[test]
fn pure_digits_classified_crawler() {
    assert!(is_crawler("1234567890"));
}

#[test]
fn punctuation_only_classified_crawler() {
    assert!(is_crawler("!@#$%^&*()"));
}

#[test]
fn input_at_buffer_limit_works() {
    let user_agent = "a".repeat(512);
    assert!(is_crawler(&user_agent));
}

#[test]
fn input_one_over_limit_returns_false() {
    let user_agent = "a".repeat(513);
    assert!(!is_crawler(&user_agent));
}

#[test]
fn oversized_with_crawler_keyword_still_false() {
    let mut user_agent = String::from("crawler ");
    user_agent.push_str(&"x".repeat(600));
    assert!(!is_crawler(&user_agent));
}

#[test]
fn oversized_browser_string_returns_false() {
    let mut user_agent = String::from("Mozilla/5.0 ");
    user_agent.push_str(&"y".repeat(600));
    assert!(!is_crawler(&user_agent));
}

#[test]
fn keyword_at_start_detected() {
    assert!(is_crawler("crawler/1.0"));
}

#[test]
fn keyword_at_end_detected() {
    assert!(is_crawler("something with bot"));
}

#[test]
fn keyword_in_middle_detected() {
    assert!(is_crawler("foo crawler bar"));
}

#[test]
fn keyword_uppercase_detected() {
    assert!(is_crawler("CRAWLER/2.0"));
}

#[test]
fn keyword_mixed_case_detected() {
    assert!(is_crawler("MyCrAwLeR/1.0"));
}

#[test]
fn whitespace_padding_does_not_break_detection() {
    assert!(is_crawler("   crawler   "));
}

#[test]
fn tab_separated_keyword_detected() {
    assert!(is_crawler("foo\tbot\tbar"));
}

#[test]
fn newline_in_input_detected() {
    assert!(is_crawler("foo\nbot\nbar"));
}

#[test]
fn ascii_boundary_chars_classified_crawler() {
    assert!(is_crawler("\x01\x02\x03"));
}

#[test]
fn high_ascii_classified_crawler() {
    assert!(is_crawler("\x7e\x7d\x7c"));
}

#[test]
fn unicode_input_handled() {
    assert!(is_crawler("ünïcödé"));
}

#[test]
fn unicode_with_crawler_keyword_detected() {
    assert!(is_crawler("ünïcödé crawler"));
}

#[test]
fn keyword_at_buffer_edge_detected() {
    let mut user_agent = "x".repeat(509);
    user_agent.push_str("bot");
    assert_eq!(user_agent.len(), 512);
    assert!(is_crawler(&user_agent));
}

#[test]
fn no_keyword_short_string_defaults_crawler() {
    assert!(is_crawler("xrawl"));
}

#[test]
fn partial_keyword_alone_no_match_defaults_crawler() {
    assert!(is_crawler("craw"));
}
