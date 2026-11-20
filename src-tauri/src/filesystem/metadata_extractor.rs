use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct ExtractedImageInfo {
    pub embedded_prompt: Option<String>,
    pub detected_keywords: Vec<String>,
    pub software: Option<String>,
}

/// Fast PNG chunk reader that extracts embedded generation prompts
/// (e.g. from Flux, ComfyUI, Automatic1111, Stable Diffusion, Midjourney, NovelAI)
/// in under 1 millisecond without running any neural networks.
pub fn extract_png_metadata(path: &Path) -> ExtractedImageInfo {
    let mut info = ExtractedImageInfo::default();

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return info,
    };

    let mut reader = BufReader::new(file);

    // Verify 8-byte PNG signature: \x89PNG\r\n\x1a\n
    let mut sig = [0u8; 8];
    if reader.read_exact(&mut sig).is_err() || sig != [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a] {
        return info;
    }

    let mut chunk_count = 0;
    while chunk_count < 100 {
        chunk_count += 1;

        let mut len_bytes = [0u8; 4];
        if reader.read_exact(&mut len_bytes).is_err() {
            break;
        }
        let length = u32::from_be_bytes(len_bytes) as usize;

        let mut type_bytes = [0u8; 4];
        if reader.read_exact(&mut type_bytes).is_err() {
            break;
        }

        let chunk_type = String::from_utf8_lossy(&type_bytes);

        if chunk_type == "IEND" {
            break;
        }

        // We only care about text chunks: tEXt, iTXt, zTXt
        if chunk_type == "tEXt" || chunk_type == "iTXt" {
            // Limit to max 64KB per text chunk to stay fast
            let to_read = length.min(64 * 1024);
            let mut data = vec![0u8; to_read];
            if reader.read_exact(&mut data).is_ok() {
                let text = String::from_utf8_lossy(&data).to_string();

                // Common keys: parameters, prompt, Description, Comment, workflow
                if text.contains("parameters")
                    || text.contains("prompt")
                    || text.contains("Description")
                    || text.contains("Comment")
                {
                    info.embedded_prompt = Some(clean_prompt_text(&text));
                }

                if text.contains("Software") || text.contains("NovelAI") || text.contains("ComfyUI") {
                    info.software = Some("AI Generated".to_string());
                }
            }

            // Skip remaining if length was larger
            if length > to_read {
                let _ = reader.seek(SeekFrom::Current((length - to_read) as i64));
            }
        } else {
            // Skip data bytes
            let _ = reader.seek(SeekFrom::Current(length as i64));
        }

        // Skip 4-byte CRC
        let _ = reader.seek(SeekFrom::Current(4));
    }

    if let Some(prompt) = &info.embedded_prompt {
        info.detected_keywords = extract_keywords_from_prompt(prompt);
    }

    info
}

fn clean_prompt_text(raw: &str) -> String {
    // Strip headers like "parameters\0", "prompt\0"
    let cleaned = raw
        .trim_start_matches("parameters\0")
        .trim_start_matches("prompt\0")
        .trim_start_matches("Description\0")
        .trim_start_matches("Comment\0")
        .trim();

    // If it contains Negative prompt or steps info, extract just the main prompt
    if let Some(pos) = cleaned.find("Negative prompt:") {
        cleaned[..pos].trim().to_string()
    } else if let Some(pos) = cleaned.find("Steps:") {
        cleaned[..pos].trim().to_string()
    } else {
        cleaned.to_string()
    }
}

pub fn extract_keywords_from_prompt(prompt: &str) -> Vec<String> {
    let common_scene_words = [
        "pyramid", "pyramids", "egypt", "waterfall", "waterfalls", "forest",
        "mountain", "mountains", "desert", "ocean", "beach", "lake", "river",
        "cyberpunk", "space", "galaxy", "planet", "sunset", "sunrise",
        "castle", "city", "futuristic", "lion", "tiger", "wolf", "dragon",
        "portrait", "landscape", "autumn", "winter", "snow", "cave", "island"
    ];

    let lower = prompt.to_lowercase();
    let mut found = Vec::new();

    for word in &common_scene_words {
        if lower.contains(word) && !found.contains(&word.to_string()) {
            found.push(word.to_string());
        }
    }

    found
}

pub fn is_valid_filename_keyword(word: &str) -> bool {
    let lower = word.to_lowercase();
    if lower.len() < 3 || lower.len() > 24 {
        return false;
    }

    // Blacklist AI generator prefix & boilerplate terms
    let blacklist = [
        "gemini", "generated", "image", "wallpaper", "wallpapers", "flux",
        "desktop", "screenshot", "download", "img", "photo", "picture", "file",
        "blueprint", "artwork"
    ];
    if blacklist.contains(&lower.as_str()) {
        return false;
    }

    // Must contain vowels
    let has_vowel = lower.chars().any(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y'));
    if !has_vowel {
        return false;
    }

    // Reject alphanumeric hash mixes (e.g. 44qyl7, a0ujc9, 6de2888c)
    let has_digit = lower.chars().any(|c| c.is_ascii_digit());
    let has_alpha = lower.chars().any(|c| c.is_ascii_alphabetic());
    if has_digit && has_alpha {
        return false;
    }

    // Reject pure digits
    if has_digit && !has_alpha {
        return false;
    }

    // Reject repetitive hash patterns (e.g. ephnhkephnhkephn, tvydfetvydfetvyd)
    if lower.len() >= 8 {
        for block_len in 3..=8 {
            if lower.len() >= block_len * 2 && lower[..block_len] == lower[block_len..block_len * 2] {
                return false;
            }
        }
    }

    // Reject excessive consonant clusters >= 5 in a row (e.g. phnhk)
    let mut consonant_count = 0;
    for c in lower.chars() {
        if matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y') {
            consonant_count = 0;
        } else if c.is_alphabetic() {
            consonant_count += 1;
            if consonant_count >= 5 {
                return false;
            }
        }
    }

    true
}
