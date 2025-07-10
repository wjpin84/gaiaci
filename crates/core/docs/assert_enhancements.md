# Enhanced Assertion Library with Custom Messages

The GaiaCI assert module has been enhanced to support optional custom error messages for all assertion functions. This improvement makes debugging failed tests much easier by providing contextual information about what went wrong.

## Key Features

✅ **Backward Compatible** - All existing code continues to work without changes  
✅ **Optional Messages** - Add custom error messages as the last parameter  
✅ **Better Debugging** - Clear, contextual error messages when assertions fail  
✅ **CI/CD Focused** - Designed for automation and pipeline validation  

## Function Signatures

All assertion functions now accept an optional message parameter:

```lua
-- Basic assertions (backward compatible)
assert.equals(actual, expected)
assert.not_nil(value)
assert.is_nil(value)
assert.is_true(condition)
assert.str_contains(haystack, needle)
assert.str_matches(text, pattern)
assert.table_has_key(table, key)

-- Enhanced assertions with custom messages
assert.equals(actual, expected, "Custom error message")
assert.not_nil(value, "Value should not be nil")
assert.is_true(condition, "Expected condition to be true")
assert.str_contains(haystack, needle, "String should contain expected text")
assert.str_matches(text, pattern, "Text should match expected format")
assert.table_has_key(table, key, "Table should contain required key")
```

## Examples

### Basic Usage (Backward Compatible)
```lua
-- These continue to work exactly as before
assert.equals(42, 42)
assert.not_nil("hello")
assert.is_true(build_success)
```

### Enhanced Usage with Custom Messages
```lua
-- CI Pipeline validation with descriptive messages
local build_result = get_build_result()

assert.is_true(build_result.success, "Build should have succeeded")
assert.equals(build_result.exit_code, 0, "Exit code should be 0 for successful build")
assert.str_contains(build_result.output, "SUCCESS", "Build output should contain success message")

-- Configuration validation
local config = load_config()
assert.table_has_key(config, "database_url", "Configuration must specify database URL")
assert.not_nil(config.api_key, "API key is required for deployment")

-- Test result validation
assert.is_true(test_count > 100, "Should have run more than 100 tests")
assert.str_matches(version, "^\\d+\\.\\d+\\.\\d+$", "Version should follow semantic versioning")
```

### Error Message Format

When an assertion fails with a custom message, the error includes both your custom context and the technical details:

```
Custom error message: Technical assertion failure details
```

For example:
```
Build should have succeeded: Assertion failed: condition is false
Configuration must specify database URL: Assertion failed: table missing key
```

## Benefits for CI/CD

1. **Faster Debugging** - Immediately understand what assertion failed and why
2. **Better Test Reports** - Clear error messages in CI logs and reports  
3. **Self-Documenting Tests** - Custom messages serve as inline documentation
4. **Easier Maintenance** - Quickly identify and fix failing pipeline steps

## Migration Guide

No migration is required! The enhancement is fully backward compatible:

- ✅ Existing code works without any changes
- ✅ Add custom messages incrementally where helpful
- ✅ Mix old and new syntax in the same codebase
- ✅ No performance impact when messages are not used

## Real-World Example

```lua
-- Comprehensive CI pipeline validation
local pipeline_result = run_ci_pipeline()

-- Build validation
assert.is_true(pipeline_result.build.success, "Build stage should complete successfully")
assert.equals(pipeline_result.build.exit_code, 0, "Build should exit with status 0")
assert.str_contains(pipeline_result.build.log, "BUILD SUCCESS", 
                   "Build log should contain success message")

-- Test validation  
assert.not_nil(pipeline_result.tests, "Test results should be available")
assert.is_true(pipeline_result.tests.passed >= 100, "At least 100 tests should pass")
assert.is_true(pipeline_result.tests.coverage > 80, "Code coverage should exceed 80%")

-- Deployment validation
assert.table_has_key(pipeline_result, "deployment", "Pipeline should include deployment stage")
assert.str_matches(pipeline_result.deployment.version, "^v\\d+\\.\\d+\\.\\d+$", 
                  "Deployment version should follow semantic versioning")

-- Artifact validation
assert.not_nil(pipeline_result.artifacts, "Build artifacts should be generated")
assert.is_true(#pipeline_result.artifacts > 0, "Should produce at least one artifact")
```

This enhancement makes GaiaCI assertions much more powerful for building robust, maintainable CI/CD pipelines!
