use crate::storage::repository::FileAnalysis;

pub fn sanitize_filename_component(raw: &str) -> String {
    let illegal = ['\\', '/', ':', '*', '?', '"', '<', '>', '|', '\'', '`', ';', '$'];
    let filtered: String = raw
        .chars()
        .map(|c| if illegal.contains(&c) || c.is_control() { '_' } else { c })
        .collect();

    // Replace multiple consecutive underscores with single underscore
    let mut result = String::new();
    let mut last_was_underscore = false;
    for c in filtered.chars() {
        if c == '_' || c == ' ' || c == '-' {
            if !last_was_underscore {
                result.push('_');
                last_was_underscore = true;
            }
        } else {
            result.push(c);
            last_was_underscore = false;
        }
    }

    result.trim_matches('_').to_string()
}

pub fn generate_deterministic_filename(
    analysis: &FileAnalysis,
    original_extension: &str,
    convention: &str,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    // 1. Style / Subcategory
    let primary_style = analysis
        .tags
        .first()
        .cloned()
        .unwrap_or_else(|| analysis.subcategory.clone());
    if !primary_style.is_empty() {
        parts.push(sanitize_filename_component(&primary_style));
    }

    // 2. Primary Subject
    let primary_subject = analysis
        .subject
        .first()
        .cloned()
        .unwrap_or_else(|| "image".to_string());
    parts.push(sanitize_filename_component(&primary_subject));

    // 3. Lighting / Environment
    if let Some(lighting) = &analysis.lighting {
        if !lighting.is_empty() {
            parts.push(sanitize_filename_component(lighting));
        }
    } else if let Some(env) = &analysis.environment {
        if !env.is_empty() {
            parts.push(sanitize_filename_component(env));
        }
    }

    // 4. Dominant color
    if let Some(color) = analysis.colors.first() {
        if !color.is_empty() {
            parts.push(sanitize_filename_component(color));
        }
    }

    let joined = if parts.is_empty() {
        "organized_image".to_string()
    } else {
        parts.join("_")
    };

    let ext = original_extension.trim_start_matches('.');
    let base_name = match convention {
        "kebab-case" => joined.replace('_', "-").to_lowercase(),
        "title_case" => {
            joined
                .split('_')
                .map(|word| {
                    let mut c = word.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str().to_lowercase().as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join("_")
        }
        _ => joined.to_lowercase(), // clean_snake_case default
    };

    format!("{}.{}", base_name, ext)
}
