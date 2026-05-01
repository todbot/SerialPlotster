use super::*;

// ── header lines ──────────────────────────────────────────────────────────

#[test]
fn header_basic() {
    assert_eq!(
        parse_line("# temp hum pres"),
        Some(LineResult::Header(vec![
            "temp".into(),
            "hum".into(),
            "pres".into()
        ]))
    );
}

#[test]
fn header_single_label() {
    assert_eq!(
        parse_line("# voltage"),
        Some(LineResult::Header(vec!["voltage".into()]))
    );
}

#[test]
fn header_empty_is_none() {
    assert_eq!(parse_line("#"), None);
    assert_eq!(parse_line("#   "), None);
}

// ── bare numeric values ───────────────────────────────────────────────────

#[test]
fn bare_comma_delimited() {
    assert_eq!(
        parse_line("1.0,2.0,3.0"),
        Some(LineResult::Data {
            values: vec![1.0, 2.0, 3.0],
            labels: None,
        })
    );
}

#[test]
fn bare_tab_delimited() {
    assert_eq!(
        parse_line("1.0\t2.0\t3.0"),
        Some(LineResult::Data {
            values: vec![1.0, 2.0, 3.0],
            labels: None,
        })
    );
}

#[test]
fn bare_space_delimited() {
    assert_eq!(
        parse_line("1.0 2.0 3.0"),
        Some(LineResult::Data {
            values: vec![1.0, 2.0, 3.0],
            labels: None,
        })
    );
}

#[test]
fn single_value() {
    assert_eq!(
        parse_line("42.0"),
        Some(LineResult::Data {
            values: vec![42.0],
            labels: None,
        })
    );
}

#[test]
fn negative_values() {
    assert_eq!(
        parse_line("-1.5,2.3,-0.0"),
        Some(LineResult::Data {
            values: vec![-1.5, 2.3, -0.0],
            labels: None,
        })
    );
}

#[test]
fn scientific_notation() {
    let r = parse_line("1e3,2.5e-1");
    assert_eq!(
        r,
        Some(LineResult::Data {
            values: vec![1000.0, 0.25],
            labels: None,
        })
    );
}

// ── label:value pairs ─────────────────────────────────────────────────────

#[test]
fn label_value_comma() {
    assert_eq!(
        parse_line("temp:23.5,hum:45.2"),
        Some(LineResult::Data {
            values: vec![23.5, 45.2],
            labels: Some(vec!["temp".into(), "hum".into()]),
        })
    );
}

#[test]
fn label_value_space() {
    assert_eq!(
        parse_line("x:1.0 y:2.0 z:3.0"),
        Some(LineResult::Data {
            values: vec![1.0, 2.0, 3.0],
            labels: Some(vec!["x".into(), "y".into(), "z".into()]),
        })
    );
}

#[test]
fn label_value_spaced_colon_space_delim() {
    assert_eq!(
        parse_line("temp: 23.5 hum: 45.2 co2: 339"),
        Some(LineResult::Data {
            values: vec![23.5, 45.2, 339.0],
            labels: Some(vec!["temp".into(), "hum".into(), "co2".into()]),
        })
    );
}

#[test]
fn label_value_spaced_colon_comma_delim() {
    assert_eq!(
        parse_line("temp: 23.5,hum: 45.2,co2: 339"),
        Some(LineResult::Data {
            values: vec![23.5, 45.2, 339.0],
            labels: Some(vec!["temp".into(), "hum".into(), "co2".into()]),
        })
    );
}

#[test]
fn label_value_single() {
    assert_eq!(
        parse_line("voltage:3.3"),
        Some(LineResult::Data {
            values: vec![3.3],
            labels: Some(vec!["voltage".into()]),
        })
    );
}

// ── invalid / dropped lines ───────────────────────────────────────────────

#[test]
fn non_numeric_text_is_none() {
    assert_eq!(parse_line("hello world"), None);
}

#[test]
fn mixed_invalid_token_drops_line() {
    assert_eq!(parse_line("1.0,foo,3.0"), None);
}

#[test]
fn empty_line_is_none() {
    assert_eq!(parse_line(""), None);
    assert_eq!(parse_line("   "), None);
}

#[test]
fn bad_label_value_drops_line() {
    assert_eq!(parse_line("temp:notanumber"), None);
}

// ── delimiter precedence ──────────────────────────────────────────────────

#[test]
fn comma_takes_precedence_over_space() {
    // "1.0,2.0" — comma wins, not split by space
    assert_eq!(
        parse_line("1.0,2.0"),
        Some(LineResult::Data {
            values: vec![1.0, 2.0],
            labels: None,
        })
    );
}

#[test]
fn tab_takes_precedence_over_space() {
    // "1.0\t2.0" — tab wins over space
    assert_eq!(
        parse_line("1.0\t2.0"),
        Some(LineResult::Data {
            values: vec![1.0, 2.0],
            labels: None,
        })
    );
}

// ── whitespace tolerance ──────────────────────────────────────────────────

#[test]
fn leading_trailing_whitespace_trimmed() {
    assert_eq!(
        parse_line("  1.0,2.0  "),
        Some(LineResult::Data {
            values: vec![1.0, 2.0],
            labels: None,
        })
    );
}

#[test]
fn spaces_around_comma_tokens() {
    assert_eq!(
        parse_line("1.0 , 2.0 , 3.0"),
        Some(LineResult::Data {
            values: vec![1.0, 2.0, 3.0],
            labels: None,
        })
    );
}

// ── Python tuple / list syntax ────────────────────────────────────────────

#[test]
fn python_tuple() {
    assert_eq!(
        parse_line("(1, 2, 3)"),
        Some(LineResult::Data {
            values: vec![1.0, 2.0, 3.0],
            labels: None,
        })
    );
}

#[test]
fn python_list() {
    assert_eq!(
        parse_line("[1.5, -2.5, 3.0]"),
        Some(LineResult::Data {
            values: vec![1.5, -2.5, 3.0],
            labels: None,
        })
    );
}

#[test]
fn python_tuple_single() {
    assert_eq!(
        parse_line("(42.0)"),
        Some(LineResult::Data {
            values: vec![42.0],
            labels: None,
        })
    );
}

#[test]
fn python_list_with_spaces() {
    assert_eq!(
        parse_line("[ 1.0 , 2.0 , 3.0 ]"),
        Some(LineResult::Data {
            values: vec![1.0, 2.0, 3.0],
            labels: None,
        })
    );
}
