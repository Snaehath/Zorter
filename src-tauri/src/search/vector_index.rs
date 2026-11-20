use std::collections::HashMap;

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}

pub fn rank_by_similarity(
    query_vector: &[f32],
    embeddings: &HashMap<String, Vec<f32>>,
    min_similarity: f32,
) -> Vec<(String, f32)> {
    let mut scores: Vec<(String, f32)> = embeddings
        .iter()
        .map(|(file_id, vec)| {
            let sim = cosine_similarity(query_vector, vec);
            (file_id.clone(), sim)
        })
        .filter(|(_, sim)| *sim >= min_similarity)
        .collect();

    // Sort descending by similarity score
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scores
}
