use crate::common::{create_test_lua, execute_lua_code};

// Test the Lua logging module directly without global state interference
// These tests verify that the Lua log functions exist and can be called without error

#[test]
fn test_log_info_integration() {
    let lua = create_test_lua().unwrap();
    
    // Test that the log functions exist and can be called without error
    let result = execute_lua_code(&lua, r#"
        log.info('hello')
        return "success"
    "#);
    
    assert!(result.is_ok(), "log.info should execute without error");
    assert_eq!(result.unwrap().to_string().unwrap(), "success");
}

#[test]
fn test_log_warn_integration() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        log.warn('hello')
        return "success"
    "#);
    
    assert!(result.is_ok(), "log.warn should execute without error");
    assert_eq!(result.unwrap().to_string().unwrap(), "success");
}

#[test]
fn test_log_error_integration() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        log.error('hello')
        return "success"
    "#);
    
    assert!(result.is_ok(), "log.error should execute without error");
    assert_eq!(result.unwrap().to_string().unwrap(), "success");
}

#[test]
fn test_log_debug_integration() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        log.debug('hello')
        return "success"
    "#);
    
    assert!(result.is_ok(), "log.debug should execute without error");
    assert_eq!(result.unwrap().to_string().unwrap(), "success");
}

#[test]
fn test_log_trace_integration() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        log.trace('hello')
        return "success"
    "#);
    
    assert!(result.is_ok(), "log.trace should execute without error");
    assert_eq!(result.unwrap().to_string().unwrap(), "success");
}

#[test]
fn test_lua_log_workflow() {
    let lua = create_test_lua().unwrap();

    let result = execute_lua_code(&lua, r#"
        -- Simulate a CI pipeline with logging
        log.info('Starting CI pipeline')
        log.debug('Loading configuration')
        log.info('Running tests')
        log.warn('Some tests were skipped')
        log.info('Building artifacts')
        log.info('Pipeline completed successfully')
        return "workflow_complete"
    "#);

    assert!(result.is_ok(), "Lua log workflow should execute without error");
    assert_eq!(result.unwrap().to_string().unwrap(), "workflow_complete");
}

#[test]
fn test_lua_log_with_variables() {
    let lua = create_test_lua().unwrap();

    let result = execute_lua_code(&lua, r#"
        local project_name = "gaiaci"
        local version = "1.0.0"
        local build_number = 42
        
        log.info('Building project: ' .. project_name)
        log.info('Version: ' .. version)
        log.debug('Build number: ' .. tostring(build_number))
        return "variables_logged"
    "#);

    assert!(result.is_ok(), "Lua log with variables should execute without error");
    assert_eq!(result.unwrap().to_string().unwrap(), "variables_logged");
}

#[test]
fn test_lua_conditional_logging() {
    let lua = create_test_lua().unwrap();

    let result = execute_lua_code(&lua, r#"
        local debug_mode = true
        local test_count = 10
        
        log.info('Starting test suite')
        
        if debug_mode then
            log.debug('Debug mode enabled')
        end
        
        if test_count > 5 then
            log.warn('Large test suite detected: ' .. tostring(test_count) .. ' tests')
        end
        
        log.info('Test suite completed')
        return "conditional_complete"
    "#);

    assert!(result.is_ok(), "Lua conditional logging should execute without error");
    assert_eq!(result.unwrap().to_string().unwrap(), "conditional_complete");
}
