use crate::storage::repository::FileAnalysis;

/// Rich text describing an analysis, used as the input for search embeddings.
pub fn build_semantic_text(analysis: &FileAnalysis) -> String {
    let subject_str = analysis.subject.join(", ");
    let style_str = analysis.tags.join(", ");
    let color_str = analysis.colors.join(", ");
    let env_str = analysis.environment.as_deref().unwrap_or("general scene");
    let light_str = analysis.lighting.as_deref().unwrap_or("natural");

    format!(
        "{} in {} style. Subject: {}. Category: {} / {}. Setting: {}. Lighting: {}. Colors: {}. Description: {}",
        analysis.visual_style,
        style_str,
        subject_str,
        analysis.category,
        analysis.subcategory,
        env_str,
        light_str,
        color_str,
        analysis.description
    )
}
