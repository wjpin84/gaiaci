use crate::common::{create_test_lua, execute_lua_code};
use std::fs;

#[test]
fn test_lua_fs_basic_operations() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Test basic fs operations
        local exists = fs.exists(".")
        local is_directory = fs.is_dir(".")
        
        return {
            current_dir_exists = exists,
            current_dir_is_dir = is_directory
        }
    "#);
    
    assert!(result.is_ok());
    let value = result.unwrap();
    
    if let mlua::Value::Table(table) = value {
        let exists: bool = table.get("current_dir_exists").unwrap();
        let is_dir: bool = table.get("current_dir_is_dir").unwrap();
        assert!(exists);
        assert!(is_dir);
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_lua_fs_file_operations() {
    let lua = create_test_lua().unwrap();
    
    // Create a temporary file for testing
    let test_file = "test_temp_file.txt";
    let test_content = "Hello from GaiaCI filesystem test!";
    
    let result = execute_lua_code(&lua, &format!(r#"
        -- Test file write and read operations
        fs.write("{}", "{}")
        
        local content = fs.read("{}")
        local exists = fs.exists("{}")
        
        return {{
            content = content,
            exists = exists
        }}
    "#, test_file, test_content, test_file, test_file));
    
    assert!(result.is_ok());
    let value = result.unwrap();
    
    if let mlua::Value::Table(table) = value {
        let content: String = table.get("content").unwrap();
        let exists: bool = table.get("exists").unwrap();
        assert_eq!(content, test_content);
        assert!(exists);
    } else {
        panic!("Expected table result");
    }
    
    // Clean up
    let _ = fs::remove_file(test_file);
}

#[test]
fn test_lua_fs_directory_operations() {
    let lua = create_test_lua().unwrap();
    
    // Create a temporary directory structure
    let test_dir = "test_temp_dir";
    fs::create_dir_all(test_dir).unwrap();
    fs::write(format!("{}/file1.txt", test_dir), "content1").unwrap();
    fs::write(format!("{}/file2.txt", test_dir), "content2").unwrap();
    
    let result = execute_lua_code(&lua, &format!(r#"
        -- Test directory operations
        local exists = fs.exists("{}")
        local is_directory = fs.is_dir("{}")
        local contents = fs.list_dir("{}")
        
        return {{
            exists = exists,
            is_dir = is_directory,
            file_count = #contents
        }}
    "#, test_dir, test_dir, test_dir));
    
    assert!(result.is_ok());
    let value = result.unwrap();
    
    if let mlua::Value::Table(table) = value {
        let exists: bool = table.get("exists").unwrap();
        let is_dir: bool = table.get("is_dir").unwrap();
        let file_count: i32 = table.get("file_count").unwrap();
        assert!(exists);
        assert!(is_dir);
        assert_eq!(file_count, 2);
    } else {
        panic!("Expected table result");
    }
    
    // Clean up
    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_lua_fs_copy_operations() {
    let lua = create_test_lua().unwrap();
    
    let source_file = "test_source.txt";
    let dest_file = "test_dest.txt";
    let test_content = "Copy test content";
    
    // Create source file
    fs::write(source_file, test_content).unwrap();
    
    let result = execute_lua_code(&lua, &format!(r#"
        -- Test file copy operation
        fs.copy("{}", "{}")
        
        local source_exists = fs.exists("{}")
        local dest_exists = fs.exists("{}")
        local dest_content = fs.read("{}")
        
        return {{
            source_exists = source_exists,
            dest_exists = dest_exists,
            content_matches = dest_content == "{}"
        }}
    "#, source_file, dest_file, source_file, dest_file, dest_file, test_content));
    
    assert!(result.is_ok());
    let value = result.unwrap();
    
    if let mlua::Value::Table(table) = value {
        let source_exists: bool = table.get("source_exists").unwrap();
        let dest_exists: bool = table.get("dest_exists").unwrap();
        let content_matches: bool = table.get("content_matches").unwrap();
        assert!(source_exists);
        assert!(dest_exists);
        assert!(content_matches);
    } else {
        panic!("Expected table result");
    }
    
    // Clean up
    let _ = fs::remove_file(source_file);
    let _ = fs::remove_file(dest_file);
}

#[test]
fn test_lua_fs_remove_operations() {
    let lua = create_test_lua().unwrap();
    
    let test_file = "test_to_remove.txt";
    let test_content = "This file will be removed";
    
    // Create file to remove
    fs::write(test_file, test_content).unwrap();
    
    let result = execute_lua_code(&lua, &format!(r#"
        -- Test file removal
        local exists_before = fs.exists("{}")
        fs.remove("{}")
        local exists_after = fs.exists("{}")
        
        return {{
            existed_before = exists_before,
            exists_after = exists_after
        }}
    "#, test_file, test_file, test_file));
    
    assert!(result.is_ok());
    let value = result.unwrap();
    
    if let mlua::Value::Table(table) = value {
        let existed_before: bool = table.get("existed_before").unwrap();
        let exists_after: bool = table.get("exists_after").unwrap();
        assert!(existed_before);
        assert!(!exists_after);
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_lua_fs_ci_pipeline_scenario() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Simulate a CI pipeline with file operations
        local project_name = "gaiaci"
        local build_dir = "build_output"
        local artifact_name = "artifact.txt"
        
        -- Create build directory
        if not fs.exists(build_dir) then
            -- In real scenario, we'd create directories, but our simple impl doesn't support mkdir
            -- so we'll just work with files
        end
        
        -- Create a build artifact
        local artifact_path = artifact_name
        local build_info = "Build: " .. project_name .. "\nTimestamp: " .. os.date()
        fs.write(artifact_path, build_info)
        
        -- Verify artifact was created
        local artifact_exists = fs.exists(artifact_path)
        local artifact_content = fs.read(artifact_path)
        
        -- Create a backup copy
        local backup_path = "backup_" .. artifact_name
        fs.copy(artifact_path, backup_path)
        local backup_exists = fs.exists(backup_path)
        
        -- Clean up original, keep backup for verification
        fs.remove(artifact_path)
        local original_removed = not fs.exists(artifact_path)
        
        -- Verify backup content
        local backup_content = fs.read(backup_path)
        local content_matches = backup_content == artifact_content
        
        -- Final cleanup
        fs.remove(backup_path)
        
        return {
            artifact_was_created = artifact_exists,
            backup_was_created = backup_exists,
            original_was_removed = original_removed,
            content_preserved = content_matches
        }
    "#);
    
    assert!(result.is_ok());
    let value = result.unwrap();
    
    if let mlua::Value::Table(table) = value {
        let artifact_created: bool = table.get("artifact_was_created").unwrap();
        let backup_created: bool = table.get("backup_was_created").unwrap();
        let original_removed: bool = table.get("original_was_removed").unwrap();
        let content_preserved: bool = table.get("content_preserved").unwrap();
        
        assert!(artifact_created);
        assert!(backup_created);
        assert!(original_removed);
        assert!(content_preserved);
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_lua_fs_error_handling() {
    let lua = create_test_lua().unwrap();
    
    // Test reading non-existent file
    let result = execute_lua_code(&lua, r#"
        -- This should fail
        local content = fs.read("/nonexistent/file/path/12345.txt")
    "#);
    
    assert!(result.is_err());
    
    // Test writing to invalid path
    let result = execute_lua_code(&lua, r#"
        -- This should also fail
        fs.write("/invalid/path/that/does/not/exist/file.txt", "content")
    "#);
    
    assert!(result.is_err());
}

#[test]
fn test_lua_fs_empty_file_operations() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Test operations with empty files
        local empty_file = "test_empty.txt"
        
        -- Write empty content
        fs.write(empty_file, "")
        
        -- Read empty content
        local content = fs.read(empty_file)
        local exists = fs.exists(empty_file)
        
        -- Clean up
        fs.remove(empty_file)
        
        return {
            content_length = #content,
            existed = exists
        }
    "#);
    
    assert!(result.is_ok());
    let value = result.unwrap();
    
    if let mlua::Value::Table(table) = value {
        let content_length: i32 = table.get("content_length").unwrap();
        let existed: bool = table.get("existed").unwrap();
        assert_eq!(content_length, 0);
        assert!(existed);
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_lua_fs_large_content_operations() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Test operations with larger content
        local large_file = "test_large.txt"
        local large_content = string.rep("Hello World! ", 1000)
        
        -- Write large content
        fs.write(large_file, large_content)
        
        -- Read it back
        local read_content = fs.read(large_file)
        local content_matches = read_content == large_content
        
        -- Clean up
        fs.remove(large_file)
        
        return {
            content_matches = content_matches,
            original_length = #large_content,
            read_length = #read_content
        }
    "#);
    
    assert!(result.is_ok());
    let value = result.unwrap();
    
    if let mlua::Value::Table(table) = value {
        let content_matches: bool = table.get("content_matches").unwrap();
        let original_length: i32 = table.get("original_length").unwrap();
        let read_length: i32 = table.get("read_length").unwrap();
        assert!(content_matches);
        assert_eq!(original_length, read_length);
        assert!(original_length > 10000); // Should be a large string
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_lua_fs_file_overwrite_operations() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Test file overwriting
        local overwrite_file = "test_overwrite.txt"
        local content1 = "Original content"
        local content2 = "New overwritten content"
        
        -- Write original content
        fs.write(overwrite_file, content1)
        local first_content = fs.read(overwrite_file)
        
        -- Overwrite with new content
        fs.write(overwrite_file, content2)
        local second_content = fs.read(overwrite_file)
        
        -- Clean up
        fs.remove(overwrite_file)
        
        return {
            first_matches = first_content == content1,
            second_matches = second_content == content2,
            different = first_content ~= second_content
        }
    "#);
    
    assert!(result.is_ok());
    let value = result.unwrap();
    
    if let mlua::Value::Table(table) = value {
        let first_matches: bool = table.get("first_matches").unwrap();
        let second_matches: bool = table.get("second_matches").unwrap();
        let different: bool = table.get("different").unwrap();
        assert!(first_matches);
        assert!(second_matches);
        assert!(different);
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_lua_fs_directory_listing_details() {
    let lua = create_test_lua().unwrap();
    
    // Create a temporary directory structure for detailed testing
    let test_dir = "test_detailed_dir";
    fs::create_dir_all(test_dir).unwrap();
    fs::write(format!("{}/file_a.txt", test_dir), "content a").unwrap();
    fs::write(format!("{}/file_b.txt", test_dir), "content b").unwrap();
    fs::write(format!("{}/file_c.log", test_dir), "log content").unwrap();
    
    let result = execute_lua_code(&lua, &format!(r#"
        -- Test detailed directory listing
        local contents = fs.list_dir("{}")
        
        -- Count different file types
        local txt_files = 0
        local log_files = 0
        local total_files = #contents
        
        for i = 1, #contents do
            local name = contents[i]
            if string.match(name, "%.txt$") then
                txt_files = txt_files + 1
            elseif string.match(name, "%.log$") then
                log_files = log_files + 1
            end
        end
        
        return {{
            total_files = total_files,
            txt_files = txt_files,
            log_files = log_files
        }}
    "#, test_dir));
    
    assert!(result.is_ok());
    let value = result.unwrap();
    
    if let mlua::Value::Table(table) = value {
        let total_files: i32 = table.get("total_files").unwrap();
        let txt_files: i32 = table.get("txt_files").unwrap();
        let log_files: i32 = table.get("log_files").unwrap();
        assert_eq!(total_files, 3);
        assert_eq!(txt_files, 2);
        assert_eq!(log_files, 1);
    } else {
        panic!("Expected table result");
    }
    
    // Clean up
    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_lua_fs_multiple_copy_chain() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Test multiple copy operations in a chain
        local original = "original.txt"
        local copy1 = "copy1.txt"
        local copy2 = "copy2.txt"
        local copy3 = "copy3.txt"
        local content = "Content to be copied multiple times"
        
        -- Create original file
        fs.write(original, content)
        
        -- Create chain of copies
        fs.copy(original, copy1)
        fs.copy(copy1, copy2)
        fs.copy(copy2, copy3)
        
        -- Verify all files have same content
        local original_content = fs.read(original)
        local copy1_content = fs.read(copy1)
        local copy2_content = fs.read(copy2)
        local copy3_content = fs.read(copy3)
        
        local all_match = (original_content == copy1_content and
                          copy1_content == copy2_content and
                          copy2_content == copy3_content and
                          copy3_content == content)
        
        -- Clean up all files
        fs.remove(original)
        fs.remove(copy1)
        fs.remove(copy2)
        fs.remove(copy3)
        
        return {
            all_content_matches = all_match
        }
    "#);
    
    assert!(result.is_ok());
    let value = result.unwrap();
    
    if let mlua::Value::Table(table) = value {
        let all_match: bool = table.get("all_content_matches").unwrap();
        assert!(all_match);
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_lua_fs_build_artifact_workflow() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Simulate a more realistic CI build artifact workflow
        local project_name = "gaiaci"
        local version = "1.0.0"
        local build_number = "42"
        
        -- Create build metadata
        local metadata = string.format("Project: %s\nVersion: %s\nBuild: %s\nDate: %s", 
                                      project_name, version, build_number, os.date())
        local metadata_file = "build-metadata.txt"
        fs.write(metadata_file, metadata)
        
        -- Create a simulated binary artifact
        local binary_content = "FAKE_BINARY_CONTENT_" .. version
        local binary_file = "gaiaci-binary"
        fs.write(binary_file, binary_content)
        
        -- Create checksums
        local metadata_checksum = "md5:" .. #metadata  -- Simplified checksum
        local binary_checksum = "md5:" .. #binary_content
        local checksum_file = "checksums.txt"
        local checksum_content = string.format("%s  %s\n%s  %s", 
                                              metadata_checksum, metadata_file,
                                              binary_checksum, binary_file)
        fs.write(checksum_file, checksum_content)
        
        -- Create release archive simulation (just copy to archive name)
        local archive_name = string.format("gaiaci-v%s-build-%s.tar.gz", version, build_number)
        fs.copy(binary_file, archive_name)
        
        -- Verify all artifacts exist
        local metadata_exists = fs.exists(metadata_file)
        local binary_exists = fs.exists(binary_file)
        local checksum_exists = fs.exists(checksum_file)
        local archive_exists = fs.exists(archive_name)
        
        -- Read back metadata to verify
        local read_metadata = fs.read(metadata_file)
        local metadata_correct = string.find(read_metadata, project_name) ~= nil
        
        -- Clean up
        fs.remove(metadata_file)
        fs.remove(binary_file)
        fs.remove(checksum_file)
        fs.remove(archive_name)
        
        return {
            metadata_exists = metadata_exists,
            binary_exists = binary_exists,
            checksum_exists = checksum_exists,
            archive_exists = archive_exists,
            metadata_correct = metadata_correct,
            workflow_complete = metadata_exists and binary_exists and checksum_exists and archive_exists
        }
    "#);
    
    assert!(result.is_ok());
    let value = result.unwrap();
    
    if let mlua::Value::Table(table) = value {
        let metadata_exists: bool = table.get("metadata_exists").unwrap();
        let binary_exists: bool = table.get("binary_exists").unwrap();
        let checksum_exists: bool = table.get("checksum_exists").unwrap();
        let archive_exists: bool = table.get("archive_exists").unwrap();
        let metadata_correct: bool = table.get("metadata_correct").unwrap();
        let workflow_complete: bool = table.get("workflow_complete").unwrap();
        
        assert!(metadata_exists);
        assert!(binary_exists);
        assert!(checksum_exists);
        assert!(archive_exists);
        assert!(metadata_correct);
        assert!(workflow_complete);
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_lua_fs_performance_simulation() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Performance test: handle multiple file operations quickly
        local start_time = os.clock()
        local test_prefix = "perf_test_"
        local file_count = 10
        
        -- Create multiple files
        for i = 1, file_count do
            local filename = test_prefix .. i .. ".txt"
            local content = string.format("Performance test file %d with some content", i)
            fs.write(filename, content)
        end
        
        -- Read all files back
        local total_content_length = 0
        for i = 1, file_count do
            local filename = test_prefix .. i .. ".txt"
            local content = fs.read(filename)
            total_content_length = total_content_length + #content
        end
        
        -- Copy all files with new names
        for i = 1, file_count do
            local source = test_prefix .. i .. ".txt"
            local dest = test_prefix .. "copy_" .. i .. ".txt"
            fs.copy(source, dest)
        end
        
        -- Clean up all files
        for i = 1, file_count do
            fs.remove(test_prefix .. i .. ".txt")
            fs.remove(test_prefix .. "copy_" .. i .. ".txt")
        end
        
        local end_time = os.clock()
        local duration = end_time - start_time
        
        return {
            files_processed = file_count * 4, -- create, read, copy, remove twice
            total_content_length = total_content_length,
            duration_seconds = duration,
            operations_completed = true
        }
    "#);
    
    assert!(result.is_ok());
    let value = result.unwrap();
    
    if let mlua::Value::Table(table) = value {
        let files_processed: i32 = table.get("files_processed").unwrap();
        let total_content_length: i32 = table.get("total_content_length").unwrap();
        let duration: f64 = table.get("duration_seconds").unwrap();
        let operations_completed: bool = table.get("operations_completed").unwrap();
        
        assert_eq!(files_processed, 40); // 10 files * 4 operations each
        assert!(total_content_length > 0);
        assert!(duration >= 0.0);
        assert!(operations_completed);
        
        // Performance should be reasonable (less than 1 second for this simple test)
        assert!(duration < 1.0, "File operations took too long: {} seconds", duration);
    } else {
        panic!("Expected table result");
    }
}
