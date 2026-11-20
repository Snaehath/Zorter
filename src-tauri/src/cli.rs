use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use clap::{Parser, Subcommand};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};

use zorter_lib::ai::ollama::OllamaClient;
use zorter_lib::ai::provider::RuntimeProvider;
use zorter_lib::ai::vision::VisionCoordinator;
use zorter_lib::core::errors::{AppError, Result};
use zorter_lib::filesystem::operations::{execute_file_operations, rollback_operation, FileOperationAction};
use zorter_lib::filesystem::scanner::scan_directory;
use zorter_lib::organization::classifier::clean_category_name;
use zorter_lib::organization::deduplication::detect_duplicates;
use zorter_lib::organization::folder_planner::create_organization_plan;
use zorter_lib::search::semantic::SemanticSearchEngine;
use zorter_lib::storage::repository::{FileAnalysis, IndexedFile, Repository};

#[derive(Parser)]
#[command(
    name = "zorter",
    about = "❖ Local AI File Intelligence — Understand. Organize. Discover.",
    version = "0.1.0"
)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<CliCommand>,

    /// Directory to organize (shortcut for: zorter organize <DIR>)
    #[arg(value_name = "DIRECTORY")]
    pub directory: Option<PathBuf>,

    /// Vision model to use (default: ministral-3:3b)
    #[arg(short, long, default_value = "ministral-3:3b")]
    pub model: String,

    /// Organization structure: 'by-category' or 'flat'
    #[arg(short, long, default_value = "by-category")]
    pub style: String,

    /// Naming convention: 'subject_scene', 'kebab-case', 'title_case'
    #[arg(short, long, default_value = "subject_scene")]
    pub naming: String,

    /// Dry run: preview planned renames without moving files
    #[arg(short, long)]
    pub dry_run: bool,

    /// Skip interactive confirmation
    #[arg(short, long)]
    pub yes: bool,

    /// Ollama endpoint
    #[arg(long, default_value = "http://127.0.0.1:11434")]
    pub endpoint: String,
}

#[derive(Subcommand)]
pub enum CliCommand {
    /// Scan directory and display file statistics and duplicates
    Scan {
        /// Directory to scan
        directory: Option<PathBuf>,
    },

    /// Analyze files with local AI vision
    Analyze {
        /// Target directory containing images
        directory: Option<PathBuf>,

        /// Vision model to use (default: ministral-3:3b)
        #[arg(short, long, default_value = "ministral-3:3b")]
        model: String,

        /// Verbose output (show full analysis card for each file)
        #[arg(short, long)]
        verbose: bool,

        /// Ollama endpoint
        #[arg(long, default_value = "http://127.0.0.1:11434")]
        endpoint: String,
    },

    /// Organize images using local AI vision model
    Organize {
        /// Target directory containing images
        directory: Option<PathBuf>,

        /// Vision model to use (default: ministral-3:3b)
        #[arg(short, long, default_value = "ministral-3:3b")]
        model: String,

        /// Organization structure: 'by-category' or 'flat'
        #[arg(short, long, default_value = "by-category")]
        style: String,

        /// Naming convention: 'subject_scene', 'kebab-case', 'title_case'
        #[arg(short, long, default_value = "subject_scene")]
        naming: String,

        /// Dry run: preview planned renames without moving files
        #[arg(short, long)]
        dry_run: bool,

        /// Skip interactive confirmation
        #[arg(short, long)]
        yes: bool,

        /// Ollama endpoint
        #[arg(long, default_value = "http://127.0.0.1:11434")]
        endpoint: String,
    },

    /// Search files semantically by natural language query
    Search {
        /// Search query
        query: String,

        /// Maximum results to show
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Ollama endpoint
        #[arg(long, default_value = "http://127.0.0.1:11434")]
        endpoint: String,
    },

    /// List available local AI models and status
    Models {
        /// Ollama endpoint
        #[arg(long, default_value = "http://127.0.0.1:11434")]
        endpoint: String,
    },

    /// Show system and index status
    Status {
        /// Ollama endpoint
        #[arg(long, default_value = "http://127.0.0.1:11434")]
        endpoint: String,
    },

    /// Roll back the last organization operation
    Undo,

    /// Launch Tauri desktop GUI
    Gui,
}

pub fn run_cli() -> Result<()> {
    let args = CliArgs::parse();

    match args.command {
        Some(CliCommand::Scan { directory }) => {
            let dir = resolve_dir(directory)?;
            cmd_scan(&dir, &args.endpoint)?;
        }
        Some(CliCommand::Analyze {
            directory,
            model,
            verbose,
            endpoint,
        }) => {
            let dir = resolve_dir(directory)?;
            cmd_analyze(&dir, &model, verbose, &endpoint)?;
        }
        Some(CliCommand::Organize {
            directory,
            model,
            style,
            naming,
            dry_run,
            yes,
            endpoint,
        }) => {
            let dir = resolve_dir(directory)?;
            cmd_organize(&dir, &model, &style, &naming, dry_run, yes, &endpoint)?;
        }
        Some(CliCommand::Search {
            query,
            limit,
            endpoint,
        }) => {
            cmd_search(&query, limit, &endpoint)?;
        }
        Some(CliCommand::Models { endpoint }) => {
            cmd_models(&endpoint)?;
        }
        Some(CliCommand::Status { endpoint }) => {
            cmd_status(&endpoint)?;
        }
        Some(CliCommand::Undo) => {
            cmd_undo()?;
        }
        Some(CliCommand::Gui) => {
            zorter_lib::run();
        }
        None => {
            if let Some(dir) = args.directory {
                cmd_organize(
                    &dir,
                    &args.model,
                    &args.style,
                    &args.naming,
                    args.dry_run,
                    args.yes,
                    &args.endpoint,
                )?;
            } else {
                cmd_menu(&args.endpoint)?;
            }
        }
    }

    Ok(())
}

fn resolve_dir(dir: Option<PathBuf>) -> Result<PathBuf> {
    match dir {
        Some(d) => Ok(d),
        None => {
            let input: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Directory to process")
                .default(".".into())
                .interact_text()
                .map_err(|e| AppError::Validation(format!("Input error: {}", e)))?;
            Ok(PathBuf::from(input))
        }
    }
}

// ============================================================================
// 1. Home / Interactive Menu (Screen 1)
// ============================================================================

fn cmd_menu(endpoint: &str) -> Result<()> {
    let ollama = OllamaClient::new(endpoint);
    let online = ollama.health_check().unwrap_or(false);

    let runtime_status = if online {
        "● Online".green().bold()
    } else {
        "● Offline".red().bold()
    };

    let vision_model = "ministral-3:3b";
    let vision_status = if online {
        "● Ready".green().bold()
    } else {
        "● Offline".dimmed()
    };

    println!();
    println!("{}", "╭─────────────────────────────────────────────────────────────╮".cyan());
    println!("{}  {}  {:<37} {} {}", "│".cyan(), "❖".cyan().bold(), "zorter".bold(), "v0.1.0".dimmed(), "│".cyan());
    println!("{}     {:<54} {}", "│".cyan(), "LOCAL AI FILE INTELLIGENCE".cyan(), "│".cyan());
    println!("{}     {:<54} {}", "│".cyan(), "Understand. Organize. Discover.".dimmed(), "│".cyan());
    println!("{}", "├─────────────────────────────────────────────────────────────┤".cyan());
    println!("{}  👁  {:<10} {:<31} {} {}", "│".cyan(), "Runtime", "Ollama", runtime_status, "│".cyan());
    println!("{}  👁  {:<10} {:<31} {} {}", "│".cyan(), "Vision", vision_model, vision_status, "│".cyan());
    println!("{}  👁  {:<10} {:<31} {} {}", "│".cyan(), "Embedding", "Gemma Embedding", vision_status, "│".cyan());
    println!("{}  💾 {:<10} {:<31} {} {}", "│".cyan(), "Storage", "JSON", "● Ready".green().bold(), "│".cyan());
    println!("{}", "╰─────────────────────────────────────────────────────────────╯".cyan());
    println!();

    let items = vec![
        "📁 Scan a directory",
        "✦  Analyze files with AI",
        "🗂  Organize (rename & categorize)",
        "🔍 Search files (semantic)",
        "📊 View status",
        "📦 View models",
        "⚙  Configure settings",
        "✕  Exit",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select action")
        .items(&items)
        .default(0)
        .interact_opt()
        .map_err(|e| AppError::Validation(format!("Menu error: {}", e)))?;

    match selection {
        Some(0) => {
            let dir = resolve_dir(None)?;
            cmd_scan(&dir, endpoint)?;
        }
        Some(1) => {
            let dir = resolve_dir(None)?;
            cmd_analyze(&dir, vision_model, true, endpoint)?;
        }
        Some(2) => {
            let dir = resolve_dir(None)?;
            cmd_organize(&dir, vision_model, "by-category", "subject_scene", false, false, endpoint)?;
        }
        Some(3) => {
            let query: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Search query")
                .interact_text()
                .map_err(|e| AppError::Validation(format!("Input error: {}", e)))?;
            cmd_search(&query, 10, endpoint)?;
        }
        Some(4) => {
            cmd_status(endpoint)?;
        }
        Some(5) => {
            cmd_models(endpoint)?;
        }
        Some(6) => {
            cmd_settings(endpoint)?;
        }
        _ => {
            println!("{}", "Goodbye!".cyan());
        }
    }

    Ok(())
}

// ============================================================================
// 2. Scanning a Directory (Screen 2)
// ============================================================================

fn cmd_scan(directory: &Path, _endpoint: &str) -> Result<()> {
    println!();
    println!("{}", "📁 ZORTER / SCAN".bold().cyan());
    println!();
    println!("{:<13} {}", "Directory".dimmed(), directory.display().to_string().cyan());
    println!("{:<13} {}", "Mode".dimmed(), "Recursive".white());
    println!("{:<13} {}", "AI".dimmed(), "Enabled".green());
    println!();

    if !directory.exists() || !directory.is_dir() {
        eprintln!("{} Directory not found: {:?}", "Error:".bold().red(), directory);
        return Err(AppError::Validation("Directory not found".to_string()));
    }

    println!("{}", "Scanning files...".dimmed());

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} [{elapsed_precise}] Discovering files...")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let cache_dir = std::env::temp_dir().join("zorter_cache");
    let files = scan_directory(directory, &cache_dir, &[], false, None, |_| {})?;
    pb.finish_and_clear();

    let mut repo = Repository::new(None)?;
    let indexed_files = repo.get_all_files();
    let indexed_ids: HashSet<String> = indexed_files.iter().map(|f| f.id.clone()).collect();
    let existing_analyses = repo.get_all_analyses();
    let analyzed_ids: HashSet<String> = existing_analyses.iter().map(|a| a.file_id.clone()).collect();

    let mut already_indexed = 0;
    let mut new_files = 0;
    let mut require_analysis = 0;

    for f in &files {
        if indexed_ids.contains(&f.id) {
            already_indexed += 1;
        } else {
            new_files += 1;
        }
        if !analyzed_ids.contains(&f.id) {
            require_analysis += 1;
        }
    }

    // Save newly scanned files to repository index
    let _ = repo.save_files(&files);

    let duplicates = detect_duplicates(&files);

    println!(
        "📁 {:<20} {:<8}  🔄 {:<18} {}",
        "Files discovered",
        files.len().to_string().bold(),
        "Already indexed",
        already_indexed.to_string().cyan()
    );
    println!(
        "🖼  {:<20} {:<8}  +  {:<18} {}",
        "Images",
        files.len().to_string().bold(),
        "New files",
        new_files.to_string().green()
    );
    println!(
        "📄 {:<20} {:<8}  ⚠️  {:<18} {}",
        "Documents",
        "0".dimmed(),
        "Duplicates",
        duplicates.len().to_string().yellow()
    );
    println!("🗃  {:<20} {:<8}", "Other", "0".dimmed());
    println!();

    println!("{}", "✓ Scan completed".bold().green());
    println!("  {} files discovered ({} images)", files.len(), files.len());
    println!("  {} files require analysis", require_analysis.to_string().yellow().bold());
    println!("  {} files already indexed", already_indexed);
    println!("  {} duplicate candidates", duplicates.len());
    println!();

    let next_cmd = format!("zorter analyze \"{}\"", directory.display());
    println!("> Next: {}", next_cmd.bold().cyan());
    println!();

    Ok(())
}

// ============================================================================
// 3. AI Analysis (Screen 3)
// ============================================================================

fn cmd_analyze(directory: &Path, model: &str, verbose: bool, endpoint: &str) -> Result<()> {
    println!();
    println!("{}", "✦ ZORTER / AI ANALYSIS".bold().cyan());
    println!();

    if !directory.exists() || !directory.is_dir() {
        eprintln!("{} Directory not found: {:?}", "Error:".bold().red(), directory);
        return Err(AppError::Validation("Directory not found".to_string()));
    }

    let ollama = Arc::new(OllamaClient::new(endpoint));
    if !ollama.health_check().unwrap_or(false) {
        eprintln!("{} Ollama is offline at {}. Run 'ollama serve' first.", "Error:".bold().red(), endpoint);
        return Err(AppError::Ollama("Ollama offline".to_string()));
    }

    let mut repo = Repository::new(None)?;
    let existing_files = repo.get_all_files();
    let cache_dir = std::env::temp_dir().join("zorter_cache");
    let files = scan_directory(directory, &cache_dir, &existing_files, false, None, |_| {})?;
    if files.is_empty() {
        println!("{}", "No supported images found to analyze.".yellow());
        return Ok(());
    }
    let _ = repo.save_files(&files);

    let existing_analyses = repo.get_all_analyses();
    let analyzed_map: HashMap<String, FileAnalysis> = existing_analyses
        .into_iter()
        .map(|a| (a.file_id.clone(), a))
        .collect();

    let files_to_analyze: Vec<&IndexedFile> = files
        .iter()
        .filter(|f| !analyzed_map.contains_key(&f.id))
        .collect();

    let total = files.len();
    let to_process = files_to_analyze.len();

    println!("{}", "╭───────────────────────────────────┬───────────────────────────────────╮".cyan());
    println!(
        "{} 👁  {:<10} {:<19} {} ⚙  {:<10} {:<19} {}",
        "│".cyan(),
        "Model:",
        model.bold().yellow(),
        "│".cyan(),
        "Runtime:",
        "Ollama".bold(),
        "│".cyan()
    );
    println!(
        "{} 📄 {:<10} {:<19} {} ⏱  {:<10} {:<19} {}",
        "│".cyan(),
        "Processing:",
        format!("{} / {}", to_process, total),
        "│".cyan(),
        "Elapsed:",
        "0s",
        "│".cyan()
    );
    println!("{}", "╰───────────────────────────────────┴───────────────────────────────────╯".cyan());
    println!();

    if to_process == 0 {
        println!("{}", "✓ All files in this directory are already analyzed!".green().bold());
        let next_cmd = format!("zorter organize \"{}\"", directory.display());
        println!("> Next: {}", next_cmd.bold().cyan());
        return Ok(());
    }

    let start_time = Instant::now();
    let vision_coord = VisionCoordinator::new(ollama, 1);
    let mut new_analyses = Vec::new();
    let mut failed = 0;

    let pb = ProgressBar::new(to_process as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.cyan} [{elapsed_precise}] [{bar:30.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
            .unwrap()
            .progress_chars("██░"),
    );

    for (idx, file) in files_to_analyze.iter().enumerate() {
        pb.set_message(format!("{}", file.current_name));

        match vision_coord.analyze_file_sync(file, model) {
            Ok(analysis) => {
                if verbose {
                    pb.suspend(|| {
                        render_analysis_card(idx + 1, to_process, file, &analysis);
                    });
                }
                let _ = repo.save_analyses(std::slice::from_ref(&analysis));
                new_analyses.push(analysis);
            }
            Err(e) => {
                failed += 1;
                pb.suspend(|| {
                    eprintln!("{} Failed on {}: {}", "Warning:".yellow(), file.current_name, e);
                });
            }
        }
        pb.inc(1);
    }

    pb.finish_and_clear();

    // Save newly generated analyses
    if !new_analyses.is_empty() {
        repo.save_analyses(&new_analyses)?;
    }

    let elapsed = start_time.elapsed();
    println!();
    println!("{}", "Overall progress".bold());
    println!(
        "[████████████████████████████████] 100%      Analyzed: {} / {}",
        new_analyses.len().to_string().green().bold(),
        to_process
    );
    println!(
        "{:<45} Remaining: {}",
        "",
        (to_process - new_analyses.len()).to_string().yellow()
    );
    println!("{:<45} Failed: {}", "", failed.to_string().red());
    println!("{:<45} Duration: {:.1}s", "", elapsed.as_secs_f32());
    println!();

    let next_cmd = format!("zorter organize \"{}\"", directory.display());
    println!("> Next: {}", next_cmd.bold().cyan());
    println!();

    Ok(())
}

fn render_analysis_card(current: usize, total: usize, file: &IndexedFile, a: &FileAnalysis) {
    println!("{}", "╭── CURRENT FILE  ──────────────────────────────────────────────────────╮".cyan());
    println!("{}  {:<68} {}", "│".cyan(), format!("{}  ({} / {})", file.current_name.bold(), current, total), "│".cyan());
    println!("{}                                                                        {}", "│".cyan(), "│".cyan());
    println!("{}  {:<68} {}", "│".cyan(), "Description".dimmed(), "│".cyan());

    // Wrap description
    let desc_lines = wrap_text(&a.description, 68);
    for l in desc_lines {
        println!("{}  {:<68} {}", "│".cyan(), l.white(), "│".cyan());
    }

    println!("{}                                                                        {}", "│".cyan(), "│".cyan());
    println!(
        "{}  {:<12} {:<19} {:<12} {:<23} {}",
        "│".cyan(),
        "Category".dimmed(),
        a.category.cyan(),
        "Style".dimmed(),
        a.visual_style.cyan(),
        "│".cyan()
    );
    println!(
        "{}  {:<12} {:<19} {:<12} {:<23} {}",
        "│".cyan(),
        "Subcategory".dimmed(),
        a.subcategory.cyan(),
        "Lighting".dimmed(),
        a.lighting.as_deref().unwrap_or("Natural").cyan(),
        "│".cyan()
    );
    println!(
        "{}  {:<12} {:<19} {:<12} {:<23} {}",
        "│".cyan(),
        "Environment".dimmed(),
        a.environment.as_deref().unwrap_or("General").cyan(),
        "Colors".dimmed(),
        a.colors.join(", ").cyan(),
        "│".cyan()
    );
    println!("{}                                                                        {}", "│".cyan(), "│".cyan());
    println!("{}  {:<68} {}", "│".cyan(), "Tags".dimmed(), "│".cyan());

    let tags_formatted = a.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join("  ");
    println!("{}  {:<68} {}", "│".cyan(), tags_formatted.cyan(), "│".cyan());
    println!("{}                                                                        {}", "│".cyan(), "│".cyan());
    println!("{}  {:<68} {}", "│".cyan(), "Suggested name".dimmed(), "│".cyan());
    println!("{}  📁 {:<65} {}", "│".cyan(), a.suggested_filename.bold().green(), "│".cyan());
    println!("{}", "╰───────────────────────────────────────────────────────────────────────╯".cyan());
    println!();
}

// ============================================================================
// 4. Organization Review & Execution (Screen 4)
// ============================================================================

fn cmd_organize(
    directory: &Path,
    _model: &str,
    style: &str,
    naming: &str,
    dry_run: bool,
    yes: bool,
    _endpoint: &str,
) -> Result<()> {
    println!();
    println!("{}", "🗂 ZORTER / ORGANIZATION PLAN".bold().cyan());
    println!();

    if !directory.exists() || !directory.is_dir() {
        eprintln!("{} Directory not found: {:?}", "Error:".bold().red(), directory);
        return Err(AppError::Validation("Directory not found".to_string()));
    }

    let mut repo = Repository::new(None)?;
    let existing_files = repo.get_all_files();
    let cache_dir = std::env::temp_dir().join("zorter_cache");
    let files = scan_directory(directory, &cache_dir, &existing_files, false, None, |_| {})?;
    if files.is_empty() {
        println!("{}", "No supported images found in directory.".yellow());
        return Ok(());
    }
    let _ = repo.save_files(&files);

    let mut analyses = repo.get_all_analyses();

    // Map existing analyses to files by directory index if file_id doesn't match
    let analysis_map: HashMap<String, FileAnalysis> =
        analyses.iter().map(|a| (a.file_id.clone(), a.clone())).collect();

    // If analyses were stored with legacy UUIDs, migrate them to deterministic file IDs
    let matched_count = files.iter().filter(|f| analysis_map.contains_key(&f.id)).count();
    if matched_count == 0 && !analyses.is_empty() {
        eprintln!("Migrating {} analyses to deterministic file IDs...", analyses.len());
        analyses.sort_by(|a, b| a.analyzed_at.cmp(&b.analyzed_at));
        for (i, a) in analyses.iter_mut().enumerate() {
            if i < files.len() {
                a.file_id = files[i].id.clone();
            }
        }
        let _ = repo.save_analyses(&analyses);
    }

    if analyses.is_empty() {
        println!("{}", "No files have been analyzed with AI vision yet.".yellow().bold());
        println!("Please run analysis first: {}", format!("zorter analyze \"{}\"", directory.display()).bold().cyan());
        return Ok(());
    }

    let org_behavior = if style == "flat" {
        "flat_rename"
    } else {
        "by_category"
    };

    let plan = create_organization_plan(directory, &files, &analyses, naming, org_behavior);

    // Tally ready vs needs review
    let mut ready_count = 0;
    let mut review_count = 0;
    let mut category_counts: HashMap<String, usize> = HashMap::new();

    for item in &plan.items {
        if item.is_analyzed && !item.category.eq_ignore_ascii_case("Needs-Review") {
            ready_count += 1;
        } else {
            review_count += 1;
        }
        let cat_label = if item.category.is_empty() {
            "Needs-Review".to_string()
        } else {
            clean_category_name(&item.category)
        };
        *category_counts.entry(cat_label).or_insert(0) += 1;
    }

    println!(
        "{} {} files analyzed    {} {} ready    {} {} need review",
        "✓".green().bold(),
        plan.items.len(),
        "✓".green().bold(),
        ready_count,
        "⚠️".yellow().bold(),
        review_count
    );
    println!();

    // Render structure & preview box
    println!("{}", "╭── PROPOSED STRUCTURE ───────────┬── PREVIEW (first 5) ────────────────╮".cyan());
    println!(
        "{} 📁 {:<27} │ {:<22} → {:<22} {}",
        "│".cyan(),
        format!("{}/", directory.file_name().and_then(|n| n.to_str()).unwrap_or("Wallpapers")).bold(),
        "Original".dimmed(),
        "Proposed".dimmed(),
        "│".cyan()
    );

    let mut cat_list: Vec<(String, usize)> = category_counts.into_iter().collect();
    cat_list.sort_by(|a, b| b.1.cmp(&a.1));

    let preview_items = &plan.items[..5.min(plan.items.len())];
    let max_rows = cat_list.len().max(preview_items.len()).max(1);

    for r in 0..max_rows {
        let left = if r < cat_list.len() {
            let (cat, count) = &cat_list[r];
            format!("   📁 {:<18} ({})", truncate(cat, 18), count)
        } else {
            String::new()
        };

        let right = if r < preview_items.len() {
            let item = &preview_items[r];
            format!("{:<22} → {:<22}", truncate(&item.original_name, 22), truncate(&item.proposed_name, 22).green().bold())
        } else {
            String::new()
        };

        println!("{} {:<32} │ {:<48} {}", "│".cyan(), left, right, "│".cyan());
    }

    println!("{}", "╰─────────────────────────────────┴─────────────────────────────────────╯".cyan());
    println!();

    if dry_run {
        println!("{}", "Dry run complete. No files were modified on disk.".yellow().bold());
        return Ok(());
    }

    if yes {
        apply_plan(&plan, &analyses, &mut repo, true)?;
        return Ok(());
    }

    let menu_options = vec![
        "📁 Apply suggested changes (organize analyzed files)",
        "🔍 Review individually",
        "📦 Include unanalyzed files into Needs-Review/",
        "💾 Export organization plan (JSON)",
        "✕  Cancel",
    ];

    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Organization Action")
        .items(&menu_options)
        .default(0)
        .interact_opt()
        .map_err(|e| AppError::Validation(format!("Select error: {}", e)))?;

    match choice {
        Some(0) => {
            apply_plan(&plan, &analyses, &mut repo, true)?;
        }
        Some(1) => {
            println!("\nDetailed File-by-File Breakdown:");
            for item in &plan.items {
                if item.is_analyzed {
                    println!(
                        "  {} -> {}\n    Destination: {}",
                        item.original_name.dimmed(),
                        item.proposed_name.green().bold(),
                        item.proposed_destination_path.cyan()
                    );
                } else {
                    println!(
                        "  {} [Unanalyzed - will remain in place]",
                        item.original_name.yellow()
                    );
                }
            }
            let confirm = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Apply changes to analyzed files now?")
                .items(&["Yes, apply changes", "No, cancel"])
                .default(0)
                .interact_opt()
                .map_err(|e| AppError::Validation(format!("Select error: {}", e)))?;

            if confirm == Some(0) {
                apply_plan(&plan, &analyses, &mut repo, true)?;
            } else {
                println!("{}", "Operation cancelled.".yellow());
            }
        }
        Some(2) => {
            apply_plan(&plan, &analyses, &mut repo, false)?;
        }
        Some(3) => {
            let json = serde_json::to_string_pretty(&plan).unwrap_or_default();
            let export_path = directory.join("zorter_plan.json");
            fs::write(&export_path, json).map_err(|e| AppError::Io(e))?;
            println!("{} Exported plan to {}", "✓".green().bold(), export_path.display().to_string().cyan());
        }
        _ => {
            println!("{}", "Operation cancelled.".yellow());
        }
    }

    Ok(())
}

fn apply_plan(
    plan: &zorter_lib::organization::folder_planner::OrganizationPlan,
    analyses: &[FileAnalysis],
    repo: &mut Repository,
    only_analyzed: bool,
) -> Result<()> {
    let actions: Vec<FileOperationAction> = plan
        .items
        .iter()
        .filter(|item| !only_analyzed || (item.is_analyzed && !item.category.eq_ignore_ascii_case("Needs-Review")))
        .map(|item| FileOperationAction {
            file_id: item.file_id.clone(),
            source_path: item.original_path.clone(),
            destination_path: item.proposed_destination_path.clone(),
            action_type: item.action_type.clone(),
        })
        .collect();

    if actions.is_empty() {
        println!("{}", "No eligible files to organize.".yellow());
        return Ok(());
    }

    let journal_entry = execute_file_operations(&actions)?;

    let _ = repo.append_operation_journal(&journal_entry);
    let _ = repo.apply_operation_updates(&journal_entry);
    let _ = repo.save_analyses(analyses);

    println!();
    println!(
        "{} Successfully organized {} files!",
        "✓".bold().green(),
        journal_entry.actions.len().to_string().bold().green()
    );
    println!("> Run {} anytime to revert these changes\n", "zorter undo".bold().yellow());

    Ok(())
}

// ============================================================================
// 5. Semantic Search (Screen 5)
// ============================================================================

fn cmd_search(query: &str, limit: usize, endpoint: &str) -> Result<()> {
    println!();
    println!("{}", "🔍 ZORTER / SEMANTIC SEARCH".bold().cyan());
    println!();
    println!("{:<13} {}", "Query:".dimmed(), query.bold().white());
    println!();

    let repo = Repository::new(None)?;
    let ollama = Arc::new(OllamaClient::new(endpoint));
    let embedding_model = repo.get_settings().preferred_embedding_model;

    println!(
        "⚙ {:<12} {:<24} ⚑ {:<10} {}",
        "Embedding:".dimmed(),
        embedding_model.cyan(),
        "Limit:".dimmed(),
        limit.to_string().bold()
    );
    println!();

    let search_engine = SemanticSearchEngine::new(ollama);
    let results = search_engine.search(&repo, &embedding_model, query, &Default::default(), limit);

    match results {
        Ok(items) if !items.is_empty() => {
            for (idx, item) in items.iter().enumerate() {
                let parent_dir = Path::new(&item.file.current_path)
                    .parent()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Root".to_string());

                println!("{}", format!("╭── {:02} ──────────────────────────────────────────────────────────╮", idx + 1).cyan());
                println!(
                    "{}  {:<45} {:<15} {}",
                    "│".cyan(),
                    item.file.current_name.bold(),
                    "Similarity".dimmed(),
                    "│".cyan()
                );
                println!(
                    "{}  📁 {:<42} {:<15} {}",
                    "│".cyan(),
                    format!("{}/", parent_dir).dimmed(),
                    format!("{:.2}", item.score).green().bold(),
                    "│".cyan()
                );
                println!("{}", "╰───────────────────────────────────────────────────────────────────╯".cyan());
            }

            println!();
            println!(
                "Showing {} of {} results             Use --limit <N> to display more.",
                items.len().to_string().green().bold(),
                items.len()
            );
        }
        _ => {
            // Fallback keyword search across description, tags, and category
            let analyses = repo.get_all_analyses();
            let q_lower = query.to_lowercase();
            let q_tokens: Vec<&str> = q_lower.split_whitespace().collect();

            let mut matched: Vec<(&FileAnalysis, f32)> = Vec::new();
            for a in &analyses {
                let text = format!("{} {} {} {} {}", a.description, a.category, a.subcategory, a.tags.join(" "), a.suggested_filename).to_lowercase();
                let mut hits = 0;
                for t in &q_tokens {
                    if text.contains(t) {
                        hits += 1;
                    }
                }
                if hits > 0 {
                    let score = 0.5 + (hits as f32 / q_tokens.len().max(1) as f32) * 0.45;
                    matched.push((a, score));
                }
            }

            matched.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            matched.truncate(limit);

            if matched.is_empty() {
                println!("{}", "No matching files found. Run 'zorter analyze <DIR>' to index files.".yellow());
            } else {
                for (idx, (a, score)) in matched.iter().enumerate() {
                    println!("{}", format!("╭── {:02} ──────────────────────────────────────────────────────────╮", idx + 1).cyan());
                    println!(
                        "{}  {:<45} {:<15} {}",
                        "│".cyan(),
                        truncate(&a.suggested_filename, 45).bold(),
                        "Similarity".dimmed(),
                        "│".cyan()
                    );
                    println!(
                        "{}  📁 {:<42} {:<15} {}",
                        "│".cyan(),
                        format!("{}/{}/", a.category, a.subcategory).dimmed(),
                        format!("{:.2}", score).green().bold(),
                        "│".cyan()
                    );
                    println!("{}", "╰───────────────────────────────────────────────────────────────────╯".cyan());
                }
                println!();
                println!("Showing {} matches.", matched.len().to_string().green().bold());
            }
        }
    }

    println!();
    Ok(())
}

// ============================================================================
// 6. Models & Status (Screen 6)
// ============================================================================

fn cmd_models(endpoint: &str) -> Result<()> {
    println!();
    println!("{}", "📦 ZORTER / MODELS".bold().cyan());
    println!();

    let ollama = OllamaClient::new(endpoint);
    print!("Connecting to Ollama at {}... ", endpoint);
    if !ollama.health_check().unwrap_or(false) {
        println!("{}", "OFFLINE".red().bold());
        return Ok(());
    }
    println!("{}", "ONLINE\n".green().bold());

    println!("{:<10} {:<32} {:<12} {}", "TYPE".dimmed(), "MODEL".dimmed(), "PROVIDER".dimmed(), "STATUS".dimmed());
    println!("{:-<68}", "");

    if let Ok(models) = ollama.fetch_models() {
        for m in models {
            let (mtype, status_badge) = if m.name.starts_with("ministral") || m.name.starts_with("qwen") || m.is_vision {
                ("👁  Vision", "● Ready".green().bold())
            } else if m.is_embedding || m.name.contains("embed") {
                ("⚙  Embed", "● Ready".green().bold())
            } else {
                ("🧠 LLM", "● Available".cyan())
            };

            println!(
                "{:<10} {:<32} {:<12} {}",
                mtype,
                truncate(&m.name, 31).bold(),
                "Ollama",
                status_badge
            );
        }
    }

    println!();
    Ok(())
}

fn cmd_status(endpoint: &str) -> Result<()> {
    println!();
    println!("{}", "📊 ZORTER / STATUS".bold().cyan());
    println!();

    let repo = Repository::new(None)?;
    let files = repo.get_all_files();
    let analyses = repo.get_all_analyses();
    let history = repo.get_operation_history().unwrap_or_default();

    let mut successful_ops = 0;
    let mut failed_ops = 0;
    for op in &history {
        if op.status == "completed" {
            successful_ops += 1;
        } else if op.status == "failed" {
            failed_ops += 1;
        }
    }

    let ollama = OllamaClient::new(endpoint);
    let online = ollama.health_check().unwrap_or(false);
    let ready_models = if online {
        ollama.fetch_models().map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    println!("📁 {:<14} {} files ({} images)", "Index", files.len().to_string().bold(), files.len());
    println!("💾 {:<14} JSON (AppData/zorter)", "Storage");
    println!("🕒 {:<14} {}", "Last scan", if files.is_empty() { "Never" } else { "Indexed" });
    println!("🕒 {:<14} {}", "Last analysis", if analyses.is_empty() { "None" } else { "Available" });
    println!(
        "⚙  {:<14} {}",
        "Models",
        if online {
            format!("{} available in Ollama", ready_models).green()
        } else {
            "Ollama offline".red()
        }
    );
    println!(
        "✓  {:<14} {} completed ({} failed)",
        "Operations",
        successful_ops.to_string().green().bold(),
        failed_ops.to_string().red()
    );

    println!();
    Ok(())
}

fn cmd_undo() -> Result<()> {
    let mut repo = Repository::new(None)?;
    let history = repo.get_operation_history()?;

    let last_completed = history
        .iter()
        .rev()
        .find(|op| op.status == "completed" && !op.operation_type.starts_with("rollback:"));

    if let Some(op) = last_completed {
        println!("Rolling back operation from {} ({} files)...", op.timestamp.yellow(), op.actions.len());
        let rollback_entry = rollback_operation(op)?;
        repo.append_operation_journal(&rollback_entry)?;
        repo.apply_operation_updates(&rollback_entry)?;
        println!("{}", "✓ Successfully rolled back all file movements and renames!".green().bold());
    } else {
        println!("{}", "No eligible operations found to undo.".yellow());
    }

    Ok(())
}

fn cmd_settings(_endpoint: &str) -> Result<()> {
    let repo = Repository::new(None)?;
    let settings = repo.get_settings();

    println!();
    println!("{}", "⚙ ZORTER / SETTINGS".bold().cyan());
    println!();
    println!("1. Preferred Vision Model:    {}", settings.preferred_vision_model.bold().yellow());
    println!("2. Preferred Embedding Model: {}", settings.preferred_embedding_model.bold().cyan());
    println!("3. Default Naming Style:      {}", settings.default_naming_convention.bold());
    println!("4. Default Org Behavior:      {}", settings.default_organization_behavior.bold());
    println!("5. Inference Concurrency:     {}", settings.inference_concurrency);
    println!();

    Ok(())
}

// ============================================================================
// Helper formatting utilities
// ============================================================================

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len && max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

fn wrap_text(s: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();

    for word in s.split_whitespace() {
        if cur.is_empty() {
            cur.push_str(word);
        } else if cur.len() + 1 + word.len() <= max_width {
            cur.push(' ');
            cur.push_str(word);
        } else {
            lines.push(cur);
            cur = word.to_string();
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}
