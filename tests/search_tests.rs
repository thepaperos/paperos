use paperos::extensions::search_backward;
use paperos::extensions::search_forward;

#[test]
fn test_search_forward_finds_match() {
    let lines: Vec<&str> = vec!["hello world", "foo bar", "hello again"];
    let result = search_forward(&lines, 0, 0, "hello");
    assert_eq!(result, Some((0, 0)));
}

#[test]
fn test_search_forward_finds_next_line() {
    let lines: Vec<&str> = vec!["aaa", "bbb foo", "ccc"];
    let result = search_forward(&lines, 0, 1, "foo");
    assert_eq!(result, Some((1, 4)));
}

#[test]
fn test_search_forward_no_match() {
    let lines: Vec<&str> = vec!["hello", "world"];
    let result = search_forward(&lines, 0, 0, "xyz");
    assert_eq!(result, None);
}

#[test]
fn test_search_backward_finds_prior_match() {
    let lines: Vec<&str> = vec!["foo bar", "baz", "qux"];
    let result = search_backward(&lines, 2, 3, "foo");
    assert_eq!(result, Some((0, 0)));
}

#[test]
fn test_search_empty_pattern_returns_none() {
    let lines: Vec<&str> = vec!["hello"];
    let result = search_forward(&lines, 0, 0, "");
    assert_eq!(result, None);
}
