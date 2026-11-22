use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use crate::filesystem::hashing::calculate_blake3_hash;
use crate::filesystem::metadata::is_supported_image;
use crate::filesystem::operations::{
    execute_file_operations, rollback_operation, validate_operation_paths, FileOperationAction,
};
use crate::organization::deduplication::detect_duplicates;
use crate::organization::naming::generate_deterministic_filename;
use crate::storage::json_store::{read_json_file, write_json_atomic};
use crate::storage::repository::{FileAnalysis, IndexedFile};

#[test]
fn test_supported_image_extensions() {
    assert!(is_supported_image(Path::new("test.png")));
    assert!(is_supported_image(Path::new("test.jpg")));
    assert!(is_supported_image(Path::new("test.JPEG")));
    assert!(is_supported_image(Path::new("test.webp")));
    assert!(is_supported_image(Path::new("test.bmp")));
    assert!(!is_supported_image(Path::new("test.txt")));
    assert!(!is_supported_image(Path::new("test.exe")));
}

#[test]
fn test_blake3_hash_consistency() {
    let temp = std::env::temp_dir().join(format!("zorter_test_{}.bin", uuid::Uuid::new_v4()));
    {
        let mut f = File::create(&temp).unwrap();
        f.write_all(b"antigravity-local-first-ai-file-organizer").unwrap();
    }

    let hash1 = calculate_blake3_hash(&temp).unwrap();
    let hash2 = calculate_blake3_hash(&temp).unwrap();
    assert_eq!(hash1, hash2);
    assert!(!hash1.is_empty());

    let _ = fs::remove_file(&temp);
}

#[test]
fn test_deterministic_naming() {
    let egypt_analysis = FileAnalysis {
        file_id: "test_egypt".to_string(),
        description: "Ancient Egyptian pyramids in Giza under the desert sun".to_string(),
        subject: vec!["pyramids".to_string(), "egypt".to_string()],
        category: "Architecture".to_string(),
        subcategory: "Egypt".to_string(),
        tags: vec!["historical".to_string(), "ancient".to_string()],
        visual_style: "Photography".to_string(),
        environment: Some("desert".to_string()),
        lighting: Some("bright daylight".to_string()),
        colors: vec!["gold".to_string(), "sand".to_string()],
        orientation: Some("landscape".to_string()),
        suggested_filename: "egypt_pyramids.png".to_string(),
        model: "qwen".to_string(),
        analyzed_at: "2026-11-20T12:00:00Z".to_string(),
    };

    let name = generate_deterministic_filename(&egypt_analysis, "png", "subject_scene");
    assert_eq!(name, "pyramids_egypt.png");

    let waterfall_analysis = FileAnalysis {
        file_id: "test_waterfall".to_string(),
        description: "A cascade waterfall flowing through a dense lush forest".to_string(),
        subject: vec!["waterfall".to_string()],
        category: "Nature".to_string(),
        subcategory: "Waterfalls".to_string(),
        tags: vec!["nature".to_string()],
        visual_style: "Photography".to_string(),
        environment: Some("forest".to_string()),
        lighting: Some("soft".to_string()),
        colors: vec!["green".to_string(), "white".to_string()],
        orientation: Some("portrait".to_string()),
        suggested_filename: "forest_waterfall.png".to_string(),
        model: "qwen".to_string(),
        analyzed_at: "2026-11-20T12:00:00Z".to_string(),
    };

    let name2 = generate_deterministic_filename(&waterfall_analysis, "png", "subject_scene");
    assert_eq!(name2, "waterfall_forest.png");

    let kebab = generate_deterministic_filename(&waterfall_analysis, "png", "kebab-case");
    assert_eq!(kebab, "waterfall-forest.png");
}

#[test]
fn test_json_atomic_write_and_recovery() {
    let temp_dir = std::env::temp_dir().join(format!("zorter_store_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();
    let file_path = temp_dir.join("test_data.json");

    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct TestData {
        message: String,
        count: u32,
    }

    let data = TestData {
        message: "Local-first persistence".to_string(),
        count: 42,
    };

    write_json_atomic(&file_path, &data).unwrap();
    assert!(file_path.exists());

    let loaded: TestData = read_json_file(&file_path).unwrap();
    assert_eq!(loaded, data);

    // Verify backup exists on second write
    let updated = TestData {
        message: "Updated persistence".to_string(),
        count: 43,
    };
    write_json_atomic(&file_path, &updated).unwrap();
    let bak_path = file_path.with_extension("json.bak");
    assert!(bak_path.exists());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_operation_execution_and_rollback() {
    let temp_dir = std::env::temp_dir().join(format!("zorter_ops_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();

    let src_path = temp_dir.join("original.png");
    let dst_path = temp_dir.join("organized").join("cyberpunk_city.png");

    File::create(&src_path).unwrap().write_all(b"fake image data").unwrap();
    assert!(src_path.exists());

    let action = FileOperationAction {
        file_id: "file_1".to_string(),
        source_path: src_path.to_string_lossy().to_string(),
        destination_path: dst_path.to_string_lossy().to_string(),
        action_type: "rename_and_move".to_string(),
    };

    // 1. Validate
    let validation = validate_operation_paths(std::slice::from_ref(&action), &[]);
    assert!(validation.valid);

    // 2. Execute
    let journal_entry = execute_file_operations(std::slice::from_ref(&action)).unwrap();
    assert!(!src_path.exists());
    assert!(dst_path.exists());

    // 3. Rollback
    let rollback_entry = rollback_operation(&journal_entry).unwrap();
    assert!(src_path.exists());
    assert!(!dst_path.exists());
    assert_eq!(rollback_entry.status, "rolled_back");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_deduplication_detection() {
    let files = vec![
        IndexedFile {
            id: "1".to_string(),
            original_name: "a.png".to_string(),
            current_name: "a.png".to_string(),
            original_path: "/a.png".to_string(),
            current_path: "/a.png".to_string(),
            file_type: "image/png".to_string(),
            size: 1000,
            modified_ms: 100,
            hash: "hash_abc".to_string(),
            thumbnail_url: None,
            indexed: true,
        },
        IndexedFile {
            id: "2".to_string(),
            original_name: "b.png".to_string(),
            current_name: "b.png".to_string(),
            original_path: "/b.png".to_string(),
            current_path: "/b.png".to_string(),
            file_type: "image/png".to_string(),
            size: 1000,
            modified_ms: 200,
            hash: "hash_abc".to_string(),
            thumbnail_url: None,
            indexed: true,
        },
        IndexedFile {
            id: "3".to_string(),
            original_name: "c.png".to_string(),
            current_name: "c.png".to_string(),
            original_path: "/c.png".to_string(),
            current_path: "/c.png".to_string(),
            file_type: "image/png".to_string(),
            size: 500,
            modified_ms: 300,
            hash: "hash_unique".to_string(),
            thumbnail_url: None,
            indexed: true,
        },
    ];

    let dups = detect_duplicates(&files);
    assert_eq!(dups.len(), 1);
    assert_eq!(dups[0].files.len(), 2);
    assert_eq!(dups[0].wasted_bytes, 1000);
}

#[test]
fn test_undoable_operation_skips_already_undone() {
    use crate::filesystem::operations::{find_undoable_operation, OperationJournalEntry};
    let entry = |id: &str, kind: &str, status: &str| OperationJournalEntry {
        id: id.into(),
        timestamp: String::new(),
        operation_type: kind.into(),
        status: status.into(),
        actions: vec![FileOperationAction {
            file_id: "f".into(),
            source_path: "a".into(),
            destination_path: "b".into(),
            action_type: "rename".into(),
        }],
    };
    let a = entry("a", "batch_organize", "completed");
    let b = entry("b", "batch_organize", "completed");
    let undo_b = entry("u", "rollback:b", "rolled_back");
    assert_eq!(find_undoable_operation(&[a.clone(), b.clone()]).unwrap().id, "b");
    assert_eq!(find_undoable_operation(&[a.clone(), b, undo_b]).unwrap().id, "a");
    let undo_a = entry("v", "rollback:a", "rolled_back");
    assert!(find_undoable_operation(&[a, undo_a]).is_none());
}

#[test]
fn test_changes_route_unanalyzed_to_needs_review_and_cleanup_folders() {
    use crate::filesystem::operations::remove_empty_folders;
    use crate::organization::changes::{build_changes, folder_counts, NEEDS_REVIEW};
    use crate::organization::folder_planner::PlanItemPreview;
    use crate::storage::repository::Repository;

    let root = std::env::temp_dir().join(format!("zorter_plan_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(root.join(NEEDS_REVIEW)).unwrap();
    // An existing file forces a collision-safe name for the unanalyzed one.
    File::create(root.join(NEEDS_REVIEW).join("x.png")).unwrap();
    let repo = Repository::new(Some(root.join(".data"))).unwrap();

    let item = |name: &str, dest: &str, analyzed: bool| PlanItemPreview {
        file_id: name.into(),
        original_name: name.into(),
        original_path: root.join(name).display().to_string(),
        proposed_name: name.into(),
        proposed_destination_path: dest.into(),
        category: String::new(),
        subcategory: String::new(),
        action_type: "rename_and_move".into(),
        thumbnail_url: None,
        included: true,
        is_analyzed: analyzed,
    };
    let ready_dest = root.join("Art").join("Sci-Fi").join("city.png").display().to_string();
    let items = vec![
        item("a.png", &ready_dest, true),
        item("x.png", "", false),
        item("same.png", &root.join("same.png").display().to_string(), true),
    ];
    let (changes, unchanged) = build_changes(&root, &items, &repo);
    assert_eq!(unchanged, 1);
    assert_eq!(changes.len(), 2);
    assert!(!changes[0].uncertain && changes[0].folder == "Art/Sci-Fi");
    assert!(changes[1].uncertain && changes[1].folder == NEEDS_REVIEW);
    assert_eq!(changes[1].proposed_name, "x_001.png");
    let folders = folder_counts(&changes);
    assert_eq!(folders.last().unwrap().name, NEEDS_REVIEW);

    // Folder cleanup removes only empty folders created by the move.
    let art = root.join("Art").join("Sci-Fi");
    fs::create_dir_all(&art).unwrap();
    let entry = crate::filesystem::operations::OperationJournalEntry {
        id: "1".into(),
        timestamp: String::new(),
        operation_type: "rollback:0".into(),
        status: "rolled_back".into(),
        actions: vec![FileOperationAction {
            file_id: "a".into(),
            source_path: root.join("a.png").display().to_string(),
            destination_path: art.join("city.png").display().to_string(),
            action_type: "rename_and_move".into(),
        }],
    };
    remove_empty_folders(&entry);
    assert!(!root.join("Art").exists());
    assert!(root.join(NEEDS_REVIEW).exists(), "non-empty folder must survive");
    fs::remove_dir_all(&root).unwrap();
}
