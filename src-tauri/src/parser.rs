#[derive(Debug, PartialEq)]
pub(crate) enum LineResult {
    /// A `# label1 label2 …` header line.
    Header(Vec<String>),
    /// A line of numeric data, optionally with inline `label:value` labels.
    Data {
        values: Vec<f64>,
        /// `Some` when every token carried a `label:` prefix; `None` for bare values.
        labels: Option<Vec<String>>,
    },
}

/// Parse one trimmed, newline-stripped line.
///
/// Delimiter precedence: `,` → `\t` → ` ` (first delimiter yielding ≥1 valid token wins).
/// Tokens may be bare floats or `label:float` pairs.
/// Any token that cannot be interpreted as `f64` discards the entire line.
pub(crate) fn parse_line(line: &str) -> Option<LineResult> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // Header line
    if let Some(rest) = line.strip_prefix('#') {
        let labels: Vec<String> = rest.split_whitespace().map(str::to_owned).collect();
        return if labels.is_empty() { None } else { Some(LineResult::Header(labels)) };
    }

    // Strip optional Python tuple/list brackets: "(1,2,3)" or "[1,2,3]"
    let line = match (line.chars().next(), line.chars().last()) {
        (Some('('), Some(')')) | (Some('['), Some(']')) => line[1..line.len() - 1].trim(),
        _ => line,
    };
    if line.is_empty() {
        return None;
    }

    // Normalise "label: value" → "label:value" so spaced colons work with any delimiter.
    let normalized;
    let line = if line.contains(": ") {
        normalized = line.replace(": ", ":");
        normalized.as_str()
    } else {
        line
    };

    // Data line — try delimiters in precedence order
    for &delim in &[',', '\t', ' '] {
        let tokens: Vec<&str> = line
            .split(delim)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if tokens.is_empty() {
            continue;
        }

        let mut values: Vec<f64> = Vec::with_capacity(tokens.len());
        let mut labels: Vec<String> = Vec::with_capacity(tokens.len());
        let mut has_labels = false;
        let mut all_valid = true;

        for token in &tokens {
            if let Some(colon) = token.find(':') {
                let label = &token[..colon];
                let val_str = &token[colon + 1..];
                match val_str.parse::<f64>() {
                    Ok(v) => {
                        values.push(v);
                        labels.push(label.to_owned());
                        has_labels = true;
                    }
                    Err(_) => {
                        all_valid = false;
                        break;
                    }
                }
            } else {
                match token.parse::<f64>() {
                    Ok(v) => {
                        values.push(v);
                        labels.push(String::new());
                    }
                    Err(_) => {
                        all_valid = false;
                        break;
                    }
                }
            }
        }

        if all_valid && !values.is_empty() {
            return Some(LineResult::Data {
                values,
                labels: if has_labels { Some(labels) } else { None },
            });
        }
    }

    None
}
