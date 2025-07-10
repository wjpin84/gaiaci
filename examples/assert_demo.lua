-- GaiaCI Assert Module with Custom Messages Demo
-- This script demonstrates the enhanced assertion functions with optional custom error messages

-- Basic assertions without custom messages (backward compatible)
gaia.log.info("Running basic assertions without custom messages...")
gaia.assert.equals(42, 42)
gaia.assert.not_nil("hello world")
gaia.assert.is_true(true)
gaia.log.info("✓ Basic assertions passed")

-- Enhanced assertions with custom messages
gaia.log.info("Running enhanced assertions with custom messages...")

-- Simulate CI pipeline data
local build_result = {
    success = true,
    exit_code = 0,
    output = "Build completed successfully in 45.2 seconds",
    tests_passed = 127,
    coverage = 85.4,
    artifacts = {"app.jar", "docs.zip", "reports.html"}
}

-- Assert build success with descriptive messages
gaia.assert.is_true(build_result.success, "Build should have succeeded")
gaia.assert.equals(build_result.exit_code, 0, "Exit code should be 0 for successful build")

-- Validate output contains success indicators
gaia.assert.str_contains(build_result.output, "successfully", "Build output should contain success message")
gaia.assert.str_matches(build_result.output, "completed successfully in [0-9]+\\.[0-9]+ seconds", 
                       "Output should match expected completion time format")

-- Validate test results
gaia.assert.not_nil(build_result.tests_passed, "Test count should not be nil")
gaia.assert.is_true(build_result.tests_passed > 100, "Should have run more than 100 tests")

-- Validate code coverage
gaia.assert.is_true(build_result.coverage > 80, "Code coverage should be above 80%")

-- Validate artifacts
gaia.assert.table_has_key(build_result, "artifacts", "Build result should include artifacts list")
gaia.assert.not_nil(build_result.artifacts, "Artifacts list should not be nil")
gaia.assert.is_true(#build_result.artifacts >= 3, "Should have at least 3 build artifacts")

-- Check specific artifacts exist
local artifacts = build_result.artifacts
gaia.assert.str_contains(table.concat(artifacts, ","), "app.jar", "Should include application JAR")
gaia.assert.str_contains(table.concat(artifacts, ","), "docs.zip", "Should include documentation")

gaia.log.info("✓ All enhanced assertions with custom messages passed")

-- Example of assertion failure with helpful message
gaia.log.warn("The following assertion will fail intentionally to show custom error message...")

-- This will fail with a descriptive error message
-- gaia.assert.str_contains(build_result.output, "FAILED", "Build output should not contain failure indicators")

gaia.log.info("Demo completed successfully!")
