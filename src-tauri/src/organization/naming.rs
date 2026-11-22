use crate::storage::repository::FileAnalysis;

const GENERIC_STOP_WORDS: &[&str] = &[
    "image",
    "photo",
    "photography",
    "picture",
    "pic",
    "wallpaper",
    "wallpapers",
    "digital",
    "art",
    "digital_art",
    "artwork",
    "visual",
    "illustration",
    "graphic",
    "graphics",
    "scene",
    "none",
    "blank",
    "general",
    "uncategorized",
    "background",
    "unknown",
];

pub fn is_generic_word(word: &str) -> bool {
    let lower = word.trim().to_lowercase();
    GENERIC_STOP_WORDS.contains(&lower.as_str())
}

pub fn sanitize_filename_component(raw: &str) -> String {
    let illegal = ['\\', '/', ':', '*', '?', '"', '<', '>', '|', '\'', '`', ';', '$', '#', '%', '&', '{', '}', '!', '@'];
    let filtered: String = raw
        .chars()
        .map(|c| if illegal.contains(&c) || c.is_control() { '_' } else { c })
        .collect();

    // Replace multiple consecutive underscores/spaces with single underscore
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

    result.trim_matches('_').to_lowercase()
}

pub fn generate_deterministic_filename(
    analysis: &FileAnalysis,
    original_extension: &str,
    convention: &str,
) -> String {
    let mut components: Vec<String> = Vec::new();

    // 1. Check suggested filename from AI first if convention is "ai_suggested"
    if convention == "ai_suggested" && !analysis.suggested_filename.is_empty() {
        let stem = std::path::Path::new(&analysis.suggested_filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let clean = sanitize_filename_component(stem);
        if !clean.is_empty() && !is_generic_word(&clean) {
            let ext = original_extension.trim_start_matches('.');
            return format!("{}.{}", clean, ext);
        }
    }

    // 2. Extract meaningful Subject keywords (e.g. "pyramids", "waterfall", "lion", "forest")
    for subj in &analysis.subject {
        let clean = sanitize_filename_component(subj);
        if !clean.is_empty() && !is_generic_word(&clean) && !components.contains(&clean) {
            components.push(clean);
            if components.len() >= 2 {
                break;
            }
        }
    }

    // 3. Extract Environment / Location (e.g. "egypt", "forest", "desert", "cliff")
    if components.len() < 2
        && let Some(env) = &analysis.environment {
            let clean_env = sanitize_filename_component(env);
            if !clean_env.is_empty() && !is_generic_word(&clean_env) && !components.contains(&clean_env) {
                components.push(clean_env);
            }
        }

    // 4. Extract Subcategory if components is still sparse (e.g. "Egypt", "Waterfalls", "Forest")
    if components.len() < 2 {
        let subcat_clean = sanitize_filename_component(&analysis.subcategory);
        if !subcat_clean.is_empty() && !is_generic_word(&subcat_clean) && !components.contains(&subcat_clean) {
            components.push(subcat_clean);
        }
    }

    // 5. Fallback to AI suggested filename if components is empty
    if components.is_empty() && !analysis.suggested_filename.is_empty() {
        let stem = std::path::Path::new(&analysis.suggested_filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let clean = sanitize_filename_component(stem);
        if !clean.is_empty() && !is_generic_word(&clean) {
            components.push(clean);
        }
    }

    // 6. Fallback to Category
    if components.is_empty() {
        let cat_clean = sanitize_filename_component(&analysis.category);
        if !cat_clean.is_empty() && !is_generic_word(&cat_clean) {
            components.push(cat_clean);
        } else {
            components.push("wallpaper".to_string());
        }
    }

    // Limit to max 2 most descriptive words for clean names like pyramids_egypt or forest_waterfall
    if components.len() > 2 && convention != "detailed" {
        components.truncate(2);
    }

    let joined = components.join("_");
    let ext = original_extension.trim_start_matches('.');

    let base_name = match convention {
        "kebab-case" => joined.replace('_', "-"),
        "title_case" => {
            joined
                .split('_')
                .map(|word| {
                    let mut c = word.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join("_")
        }
        _ => joined, // clean_snake_case (e.g. pyramids_egypt, waterfalls, forest)
    };

    format!("{}.{}", base_name, ext)
}
