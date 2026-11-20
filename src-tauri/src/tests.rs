#[cfg(test)]
mod tests {
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
        let validation = validate_operation_paths(&[action.clone()], &[]);
        assert!(validation.valid);

        // 2. Execute
        let journal_entry = execute_file_operations(&[action.clone()]).unwrap();
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
    fn test_filename_hash_rejection() {
        use crate::filesystem::metadata_extractor::is_valid_filename_keyword;
        // Gibberish hashes should be rejected
        assert!(!is_valid_filename_keyword("ephnhkephnhkephn"));
        assert!(!is_valid_filename_keyword("tvydfetvydfetvyd"));
        assert!(!is_valid_filename_keyword("44qy1744qy1744qy"));
        assert!(!is_valid_filename_keyword("a0ujc9a0ujc9a0uj"));
        assert!(!is_valid_filename_keyword("gemini"));
        assert!(!is_valid_filename_keyword("generated"));
        assert!(!is_valid_filename_keyword("image"));

        // Valid keywords should be accepted
        assert!(is_valid_filename_keyword("waterfall"));
        assert!(is_valid_filename_keyword("forest"));
        assert!(is_valid_filename_keyword("egypt"));
        assert!(is_valid_filename_keyword("greek"));
        assert!(is_valid_filename_keyword("world"));
        assert!(is_valid_filename_keyword("pyramid"));
        assert!(is_valid_filename_keyword("paris"));
    }
}
