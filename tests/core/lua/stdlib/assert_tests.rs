use crate::common::{create_test_lua, execute_lua_code};

#[test]
fn test_lua_assert_equals_integration() {
    let lua = create_test_lua().unwrap();
    
    // Test successful assertion
    let result = execute_lua_code(&lua, r#"
        assert.equals(42, 42)
        return "success"
    "#);
    assert!(result.is_ok());
    
    // Test failed assertion
    let result = execute_lua_code(&lua, r#"
        assert.equals(42, 43)
    "#);
    assert!(result.is_err());
}

#[test]
fn test_lua_string_assertions_workflow() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        local text = "Hello Rust World"
        assert.str_contains(text, "Rust")
        assert.str_matches(text, "^Hello.*World$")
        return true
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_table_assertions() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        local config = {
            name = "test",
            version = "1.0.0"
        }
        assert.table_has_key(config, "name")
        assert.equals(config.name, "test")
        return true
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_complex_assertion_workflow() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Test a realistic CI scenario
        local build_info = {
            status = "success",
            artifacts = {"binary", "docs"},
            commit_hash = "abc123def"
        }
        
        -- Validate build info
        assert.not_nil(build_info)
        assert.table_has_key(build_info, "status")
        assert.equals(build_info.status, "success")
        assert.str_matches(build_info.commit_hash, "^[a-f0-9]+$")
        
        return "All assertions passed"
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_nil_assertions() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Test nil assertion
        assert.is_nil(nil)
        
        -- Test not_nil assertion  
        assert.not_nil("hello")
        
        return true
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_boolean_assertions() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Test boolean assertions
        assert.is_true(true)
        
        -- Test with expressions
        assert.is_true(5 > 3)
        assert.is_true("hello" == "hello")
        
        return true
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_error_propagation() {
    let lua = create_test_lua().unwrap();
    
    // Test that Lua errors are properly propagated
    let result = execute_lua_code(&lua, r#"
        assert.equals("hello", "world")
    "#);
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Assertion failed"));
}

#[test]
fn test_regex_assertion_scenarios() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Test various regex patterns
        local version = "v1.2.3"
        assert.str_matches(version, "^v\\d+\\.\\d+\\.\\d+$")
        
        local email = "test@example.com"
        assert.str_matches(email, ".+@.+\\..+")
        
        local hash = "abc123def456"
        assert.str_matches(hash, "^[a-f0-9]+$")
        
        return true
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_nested_table_assertions() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        local config = {
            database = {
                host = "localhost",
                port = 5432
            },
            cache = {
                enabled = true,
                ttl = 3600
            }
        }
        
        -- Test nested structure
        assert.table_has_key(config, "database")
        assert.table_has_key(config.database, "host")
        assert.equals(config.database.host, "localhost")
        assert.is_true(config.cache.enabled)
        
        return true
    "#);
    
    assert!(result.is_ok());
}
