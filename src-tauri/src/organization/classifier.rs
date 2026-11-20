pub fn clean_category_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "Uncategorized".to_string();
    }

    // Capitalize first letter of each word
    let sanitized: String = trimmed
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' })
        .collect();

    let words: Vec<String> = sanitized
        .split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str(),
            }
        })
        .collect();

    if words.is_empty() {
        "Uncategorized".to_string()
    } else {
        words.join(" ")
    }
}
