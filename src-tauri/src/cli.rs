use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

use zorter_lib::ai::ollama::OllamaClient;
use zorter_lib::ai::provider::RuntimeProvider;
use zorter_lib::ai::vision::VisionCoordinator;
use zorter_lib::core::errors::{AppError, Result};
use zorter_lib::filesystem::operations::{execute_file_operations, rollback_operation, FileOperationAction};
use zorter_lib::filesystem::scanner::scan_directory;
use zorter_lib::organization::deduplication::detect_duplicates;
use zorter_lib::organization::folder_planner::create_organization_plan;
use zorter_lib::storage::repository::Repository;

#[derive(Parser)]
#[command(
    name = "zorter",
    about = "⚡ Local-First AI Wallpaper & File Organizer",
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
    /// Organize images using local AI vision model
    Organize {
        /// Target directory containing images
        directory: PathBuf,

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

    /// Scan directory and display file statistics and duplicates
    Scan {
        /// Directory to scan
        directory: PathBuf,
    },

    /// Roll back the last organization operation
    Undo,

    /// Show status of Ollama connection and available models
    Status {
        /// Ollama endpoint
        #[arg(long, default_value = "http://127.0.0.1:11434")]
        endpoint: String,
    },

    /// Launch Tauri desktop GUI
    Gui,
}

pub fn run_cli() -> Result<()> {
    let args = CliArgs::parse();

    match args.command {
        Some(CliCommand::Organize {
            directory,
            model,
            style,
            naming,
            dry_run,
            yes,
            endpoint,
        }) => {
            cmd_organize(&directory, &model, &style, &naming, dry_run, yes, &endpoint)?;
        }
        Some(CliCommand::Scan { directory }) => {
            cmd_scan(&directory)?;
        }
        Some(CliCommand::Undo) => {
            cmd_undo()?;
        }
        Some(CliCommand::Status { endpoint }) => {
            cmd_status(&endpoint)?;
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
                println!("{}", "⚡ Zorter — Local-First AI File Organizer".bold().cyan());
                println!("Usage: zorter <DIRECTORY> [OPTIONS]");
                println!("   or: zorter organize <DIRECTORY> [OPTIONS]");
                println!("   or: zorter undo");
                println!("   or: zorter status\n");
                println!("Run 'zorter --help' for all options.");
            }
        }
    }

    Ok(())
}

fn cmd_organize(
    directory: &Path,
    model: &str,
    style: &str,
    naming: &str,
    dry_run: bool,
    yes: bool,
    endpoint: &str,
) -> Result<()> {
    println!("\n{}", "========================================================".cyan());
    println!("{}", "  ⚡ Zorter Local AI File Organizer".bold().cyan());
    println!("{}\n", "========================================================".cyan());

    if !directory.exists() || !directory.is_dir() {
        eprintln!("{} Directory not found: {:?}", "Error:".bold().red(), directory);
        return Err(AppError::Validation("Directory not found".to_string()));
    }

    let ollama = Arc::new(OllamaClient::new(endpoint));
    print!("Connecting to Ollama at {}... ", endpoint);
    io::stdout().flush().unwrap();

    if !ollama.health_check().unwrap_or(false) {
        println!("{}", "OFFLINE".bold().red());
        eprintln!(
            "\n{} Ollama is not running at {}.\nPlease run 'ollama serve' first.",
            "Error:".bold().red(),
            endpoint
        );
        return Err(AppError::Ollama("Ollama offline".to_string()));
    }
    println!("{}", "CONNECTED".bold().green());
    println!("Vision Model: {}\n", model.bold().yellow());

    // 1. Scan directory
    println!("Scanning {:?} for supported images...", directory);
    let cache_dir = std::env::temp_dir().join("zorter_cache");
    let files = scan_directory(directory, &cache_dir, &[], false, None, |_| {})?;

    if files.is_empty() {
        println!("{}", "No supported images found in directory.".yellow());
        return Ok(());
    }

    println!("Found {} images to analyze.\n", files.len().to_string().bold().green());

    let vision_coord = VisionCoordinator::new(ollama, 1);
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:30.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut analyses = Vec::new();
    let mut failed = 0;

    for file in &files {
        pb.set_message(format!("{}", file.current_name));
        match vision_coord.analyze_file_sync(file, model) {
            Ok(analysis) => {
                analyses.push(analysis);
            }
            Err(e) => {
                failed += 1;
                eprintln!("\n{} Failed on {}: {}", "Warning:".yellow(), file.current_name, e);
            }
        }
        pb.inc(1);
    }
    pb.finish_with_message("AI analysis complete!");

    println!("\nAnalyzed: {} | Failed: {}", analyses.len().to_string().green(), failed.to_string().red());

    // 2. Generate Plan
    let org_behavior = if style == "flat" {
        "flat_rename"
    } else {
        "by_category"
    };

    let plan = create_organization_plan(directory, &files, &analyses, naming, org_behavior);

    // 3. Print Plan Preview Table
    println!("\n{}", "Proposed Organization Plan:".bold().underline());
    println!("{:<45} {:<35} {:<40}", "Original File".bold(), "New Filename".bold(), "Destination".bold());
    println!("{:-<125}", "");

    for item in &plan.items {
        let dest_display = Path::new(&item.proposed_destination_path)
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Root".to_string());

        let short_orig = if item.original_name.len() > 42 {
            format!("{}...", &item.original_name[..39])
        } else {
            item.original_name.clone()
        };

        println!(
            "{:<45} -> {:<32} in [{}]",
            short_orig.dimmed(),
            item.proposed_name.bold().green(),
            dest_display.cyan()
        );
    }

    println!("{:-<125}", "");
    println!("Total files to organize: {}\n", plan.items.len().to_string().bold());

    if dry_run {
        println!("{}", "Dry run complete. No files were modified.".yellow().bold());
        return Ok(());
    }

    // 4. Confirmation
    if !yes {
        print!("Apply these changes? [y/N]: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let trimmed = input.trim().to_lowercase();
        if trimmed != "y" && trimmed != "yes" {
            println!("{}", "Operation cancelled by user.".yellow());
            return Ok(());
        }
    }

    // 5. Execute operations
    let actions: Vec<FileOperationAction> = plan
        .items
        .into_iter()
        .map(|item| FileOperationAction {
            file_id: item.file_id,
            source_path: item.original_path,
            destination_path: item.proposed_destination_path,
            action_type: item.action_type,
        })
        .collect();

    let journal_entry = execute_file_operations(&actions)?;

    // Save journal to AppData repository history
    let mut repo = Repository::new(None)?;
    let _ = repo.append_operation_journal(&journal_entry);
    let _ = repo.apply_operation_updates(&journal_entry);
    let _ = repo.save_analyses(&analyses);

    println!(
        "\n{} Successfully organized {} files!",
        "✓".bold().green(),
        journal_entry.actions.len().to_string().bold().green()
    );
    println!("(Run {} anytime to revert these changes)\n", "zorter undo".bold().yellow());

    Ok(())
}

fn cmd_scan(directory: &Path) -> Result<()> {
    println!("\nScanning {:?}...", directory);
    let cache_dir = std::env::temp_dir().join("zorter_cache");
    let files = scan_directory(directory, &cache_dir, &[], false, None, |_| {})?;

    println!("Found {} supported images.", files.len().to_string().green());
    let total_size: u64 = files.iter().map(|f| f.size).sum();
    println!("Total size: {:.2} MB", total_size as f64 / 1_048_576.0);

    let duplicates = detect_duplicates(&files);
    if duplicates.is_empty() {
        println!("{}", "✓ No duplicate files detected.".green());
    } else {
        println!(
            "{} Detected {} duplicate group(s) wasting {:.2} MB:",
            "!".yellow().bold(),
            duplicates.len(),
            duplicates.iter().map(|d| d.wasted_bytes).sum::<u64>() as f64 / 1_048_576.0
        );
        for (i, group) in duplicates.iter().enumerate() {
            println!("  Group {}: {} identical files", i + 1, group.files.len());
            for f in &group.files {
                println!("    - {}", f.current_path.dimmed());
            }
        }
    }

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

fn cmd_status(endpoint: &str) -> Result<()> {
    println!("\n{}", "⚡ Zorter Status Report".bold().cyan());
    println!("{:-<40}", "");

    let ollama = OllamaClient::new(endpoint);
    print!("Ollama Connection ({}): ", endpoint);
    if ollama.health_check().unwrap_or(false) {
        println!("{}", "ONLINE".green().bold());
        if let Ok(models) = ollama.fetch_models() {
            println!("\nAvailable Local Models:");
            for m in models {
                let badge = if m.is_vision { "[VISION]".green() } else if m.is_embedding { "[EMBED]".blue() } else { "[LLM]".dimmed() };
                println!("  - {:<30} {:.1} GB {}", m.name, (m.size_bytes as f64) / 1_073_741_824.0, badge);
            }
        }
    } else {
        println!("{}", "OFFLINE".red().bold());
        println!("Make sure Ollama is running ('ollama serve').");
    }

    let repo = Repository::new(None)?;
    let files = repo.get_all_files();
    let analyses = repo.get_all_analyses();
    println!("\nDatabase Index:");
    println!("  Indexed Files:    {}", files.len());
    println!("  Stored Analyses:  {}", analyses.len());
    println!();

    Ok(())
}
