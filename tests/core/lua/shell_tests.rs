use crate::common::{create_test_lua, execute_lua_code};

#[test]
fn test_lua_shell_basic_command() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        local result = shell.run({
            cmd = "echo 'Hello from shell'",
            capture = true
        })
        
        assert.not_nil(result)
        assert.table_has_key(result, "stdout")
        assert.table_has_key(result, "success")
        assert.equals(result.success, true)
        
        return result
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_shell_with_environment() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        local result = shell.run({
            cmd = "echo $TEST_VAR",
            capture = true,
            env = {
                TEST_VAR = "test_value"
            }
        })
        
        assert.equals(result.success, true)
        assert.str_contains(result.stdout, "test_value")
        
        return result
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_shell_error_handling() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        local result = shell.run({
            cmd = "false",  -- Command that always fails
            capture = true
        })
        
        assert.not_nil(result)
        assert.equals(result.success, false)
        assert.not_nil(result.code)
        
        return result
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_ci_pipeline_simulation() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Simulate a CI pipeline step
        local test_result = shell.run({
            cmd = "echo 'Running tests...' && echo 'All tests passed'",
            capture = true
        })
        
        -- Validate the test step
        assert.equals(test_result.success, true)
        assert.str_contains(test_result.stdout, "tests passed")
        
        -- Simulate build step
        local build_result = shell.run({
            cmd = "echo 'Building project...' && echo 'Build successful'",
            capture = true
        })
        
        assert.equals(build_result.success, true)
        assert.str_contains(build_result.stdout, "Build successful")
        
        return {
            test_passed = test_result.success,
            build_passed = build_result.success
        }
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_shell_working_directory() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        local result = shell.run({
            cmd = "pwd",
            capture = true,
            cwd = "/tmp"
        })
        
        assert.equals(result.success, true)
        assert.str_contains(result.stdout, "tmp")
        
        return result
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_shell_multiple_environment_variables() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        local result = shell.run({
            cmd = "echo $VAR1 $VAR2 $VAR3",
            capture = true,
            env = {
                VAR1 = "first",
                VAR2 = "second", 
                VAR3 = "third"
            }
        })
        
        assert.equals(result.success, true)
        assert.str_contains(result.stdout, "first")
        assert.str_contains(result.stdout, "second")
        assert.str_contains(result.stdout, "third")
        
        return result
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_shell_stderr_capture() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        local result = shell.run({
            cmd = "echo 'stdout message' && echo 'stderr message' >&2",
            capture = true
        })
        
        assert.equals(result.success, true)
        assert.str_contains(result.stdout, "stdout message")
        assert.str_contains(result.stderr, "stderr message")
        
        return result
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_shell_no_capture_mode() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        local result = shell.run({
            cmd = "echo 'This output will not be captured'",
            capture = false
        })
        
        assert.equals(result.success, true)
        assert.equals(result.code, 0)
        -- stdout and stderr should not be present when capture=false
        assert.is_nil(result.stdout)
        assert.is_nil(result.stderr)
        
        return result
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_shell_custom_shell() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        local result = shell.run({
            cmd = "echo 'Using custom shell'",
            capture = true,
            shell = "/bin/bash"
        })
        
        assert.equals(result.success, true)
        assert.str_contains(result.stdout, "Using custom shell")
        
        return result
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_shell_complex_command_chain() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        local result = shell.run({
            cmd = "echo 'step1' && echo 'step2' && echo 'step3'",
            capture = true
        })
        
        assert.equals(result.success, true)
        assert.str_contains(result.stdout, "step1")
        assert.str_contains(result.stdout, "step2")
        assert.str_contains(result.stdout, "step3")
        
        return result
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_shell_pipeline_with_files() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Create a test file, process it, and clean up
        local write_result = shell.run({
            cmd = "echo 'test content' > shell_test_file.txt",
            capture = true
        })
        
        assert.equals(write_result.success, true)
        
        local read_result = shell.run({
            cmd = "cat shell_test_file.txt",
            capture = true
        })
        
        assert.equals(read_result.success, true)
        assert.str_contains(read_result.stdout, "test content")
        
        local cleanup_result = shell.run({
            cmd = "rm shell_test_file.txt",
            capture = true
        })
        
        assert.equals(cleanup_result.success, true)
        
        return {
            write_success = write_result.success,
            read_success = read_result.success,
            cleanup_success = cleanup_result.success,
            content_correct = string.find(read_result.stdout, "test content") ~= nil
        }
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_shell_build_system_simulation() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Simulate a complete build pipeline with simpler commands
        local project_name = "gaiaci"
        local build_env = {
            PROJECT_NAME = project_name,
            BUILD_TYPE = "release",
            BUILD_NUMBER = "123"
        }
        
        -- Step 1: Clean
        local clean_result = shell.run({
            cmd = "echo 'Cleaning project build'",
            capture = true,
            env = build_env
        })
        
        -- Step 2: Configure  
        local config_result = shell.run({
            cmd = "echo 'Configuring project for release'",
            capture = true,
            env = build_env
        })
        
        -- Step 3: Build
        local build_result = shell.run({
            cmd = "echo 'Building project' && echo 'Build completed successfully'",
            capture = true,
            env = build_env
        })
        
        -- Step 4: Test
        local test_result = shell.run({
            cmd = "echo 'Running tests' && echo 'All tests passed'",
            capture = true,
            env = build_env
        })
        
        -- Validate all steps
        assert.equals(clean_result.success, true)
        assert.equals(config_result.success, true)
        assert.equals(build_result.success, true)
        assert.equals(test_result.success, true)
        
        -- Check specific outputs
        assert.str_contains(clean_result.stdout, "Cleaning")
        assert.str_contains(config_result.stdout, "release")
        assert.str_contains(build_result.stdout, "completed successfully")
        assert.str_contains(test_result.stdout, "tests passed")
        
        return {
            pipeline_success = clean_result.success and config_result.success and 
                             build_result.success and test_result.success,
            steps_completed = 4
        }
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_shell_conditional_execution() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Test conditional execution based on previous command results
        local check_result = shell.run({
            cmd = "echo 'condition check passed'",
            capture = true
        })
        
        local next_step = nil
        if check_result.success then
            next_step = shell.run({
                cmd = "echo 'Executing next step because condition passed'",
                capture = true
            })
        else
            next_step = shell.run({
                cmd = "echo 'Skipping next step because condition failed'",
                capture = true
            })
        end
        
        assert.equals(check_result.success, true)
        assert.equals(next_step.success, true)
        assert.str_contains(next_step.stdout, "Executing next step")
        
        return {
            condition_passed = check_result.success,
            next_step_executed = next_step.success
        }
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_shell_error_recovery() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Test error recovery and fallback execution
        local failing_result = shell.run({
            cmd = "false",  -- This will fail
            capture = true
        })
        
        assert.equals(failing_result.success, false)
        
        -- Recovery action
        local recovery_result = shell.run({
            cmd = "echo 'Recovered from failure'",
            capture = true
        })
        
        assert.equals(recovery_result.success, true)
        assert.str_contains(recovery_result.stdout, "Recovered from failure")
        
        return {
            initial_failed = not failing_result.success,
            recovery_succeeded = recovery_result.success
        }
    "#);
    
    assert!(result.is_ok());
}

#[test]
fn test_lua_shell_performance_multiple_commands() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Test performance with multiple quick commands
        local start_time = os.clock()
        local command_count = 5
        local successful_commands = 0
        
        for i = 1, command_count do
            local result = shell.run({
                cmd = "echo 'Command " .. i .. " executed'",
                capture = true
            })
            if result.success then
                successful_commands = successful_commands + 1
            end
        end
        
        local end_time = os.clock()
        local duration = end_time - start_time
        
        return {
            commands_executed = command_count,
            successful_commands = successful_commands,
            all_succeeded = successful_commands == command_count,
            duration_seconds = duration,
            performance_acceptable = duration < 2.0  -- Should be fast
        }
    "#);
    
    assert!(result.is_ok());
    
    if let mlua::Value::Table(table) = result.unwrap() {
        let all_succeeded: bool = table.get("all_succeeded").unwrap();
        let performance_acceptable: bool = table.get("performance_acceptable").unwrap();
        assert!(all_succeeded);
        assert!(performance_acceptable);
    }
}

#[test]
fn test_lua_shell_comprehensive_ci_workflow() {
    let lua = create_test_lua().unwrap();
    
    let result = execute_lua_code(&lua, r#"
        -- Comprehensive CI workflow test
        local workflow_steps = {}
        
        -- Step 1: Environment setup
        local env_setup = shell.run({
            cmd = "echo 'Setting up CI environment'",
            capture = true,
            env = {
                CI = "true",
                BUILD_ENV = "production"
            }
        })
        table.insert(workflow_steps, env_setup.success)
        
        -- Step 2: Source code checkout simulation
        local checkout = shell.run({
            cmd = "echo 'Checking out source code from repository'",
            capture = true
        })
        table.insert(workflow_steps, checkout.success)
        
        -- Step 3: Dependency installation
        local deps = shell.run({
            cmd = "echo 'Installing dependencies...' && echo 'Dependencies installed successfully'",
            capture = true
        })
        table.insert(workflow_steps, deps.success)
        
        -- Step 4: Code quality checks
        local quality = shell.run({
            cmd = "echo 'Running code quality checks...' && echo 'Quality checks passed'",
            capture = true
        })
        table.insert(workflow_steps, quality.success)
        
        -- Step 5: Unit tests
        local unit_tests = shell.run({
            cmd = "echo 'Running unit tests...' && echo 'Unit tests: 25 passed, 0 failed'",
            capture = true
        })
        table.insert(workflow_steps, unit_tests.success)
        
        -- Step 6: Integration tests
        local integration_tests = shell.run({
            cmd = "echo 'Running integration tests...' && echo 'Integration tests: 10 passed, 0 failed'",
            capture = true
        })
        table.insert(workflow_steps, integration_tests.success)
        
        -- Step 7: Build
        local build = shell.run({
            cmd = "echo 'Building project...' && echo 'Build completed: gaiaci-v1.0.0'",
            capture = true
        })
        table.insert(workflow_steps, build.success)
        
        -- Step 8: Package
        local package = shell.run({
            cmd = "echo 'Creating release package...' && echo 'Package created: gaiaci-v1.0.0.tar.gz'",
            capture = true
        })
        table.insert(workflow_steps, package.success)
        
        -- Count successful steps
        local successful_steps = 0
        for i, success in ipairs(workflow_steps) do
            if success then
                successful_steps = successful_steps + 1
            end
        end
        
        return {
            total_steps = #workflow_steps,
            successful_steps = successful_steps,
            workflow_success = successful_steps == #workflow_steps,
            unit_test_output = unit_tests.stdout,
            build_output = build.stdout
        }
    "#);
    
    assert!(result.is_ok());
    
    if let mlua::Value::Table(table) = result.unwrap() {
        let workflow_success: bool = table.get("workflow_success").unwrap();
        let total_steps: i32 = table.get("total_steps").unwrap();
        let successful_steps: i32 = table.get("successful_steps").unwrap();
        
        assert!(workflow_success);
        assert_eq!(total_steps, 8);
        assert_eq!(successful_steps, 8);
    }
}    #[test]
    fn test_lua_shell_environment_detection() {
        let lua = create_test_lua().unwrap();        let script = r#"
            -- Test all environment detection functions
            local pwd_result = shell.pwd()
            local shell_result = shell.shell()
            local os_result = shell.os()
            local arch_result = shell.arch()
            
            -- Verify results are non-empty strings
            assert.not_nil(pwd_result, "pwd should not be nil")
            assert.not_nil(shell_result, "shell should not be nil")
            assert.not_nil(os_result, "os should not be nil")
            assert.not_nil(arch_result, "arch should not be nil")
            
            return {
                pwd = pwd_result,
                shell = shell_result,
                os = os_result,
                arch = arch_result
            }
        "#;
    
    let result: mlua::Table = lua.load(script).eval().expect("Environment detection script should succeed");
    
    let pwd: String = result.get("pwd").unwrap();
    let shell: String = result.get("shell").unwrap();
    let os: String = result.get("os").unwrap();
    let arch: String = result.get("arch").unwrap();
    
    // Verify pwd returns an absolute path
    assert!(pwd.starts_with('/') || (pwd.len() >= 3 && pwd.chars().nth(1) == Some(':')));
    
    // Verify shell is a known type
    let known_shells = vec!["bash", "zsh", "sh", "fish", "cmd", "pwsh", "powershell", "unknown"];
    assert!(known_shells.contains(&shell.as_str()));
    
    // Verify OS is a known type
    let known_os = vec!["linux", "windows", "macos", "freebsd", "openbsd", "netbsd", "unknown"];
    assert!(known_os.contains(&os.as_str()));
    
    // Verify architecture is a known type
    let known_archs = vec!["x86_64", "aarch64", "x86", "arm", "riscv64", "unknown"];
    assert!(known_archs.contains(&arch.as_str()));
}    #[test]
    fn test_lua_shell_which_command() {
        let lua = create_test_lua().unwrap();        let script = r#"
            -- Test the which function
            local test_cmd = "NONEXISTENT_COMMAND_12345"
            
            -- Test with a command that should exist
            local shell_cmd = "sh"  -- Should exist on Unix systems
            if shell.os() == "windows" then
                shell_cmd = "cmd"  -- Use cmd on Windows
            end
            
            local success, path = pcall(function()
                return shell.which(shell_cmd)
            end)
            
            -- Test with a command that shouldn't exist
            local fail_success, fail_result = pcall(function()
                return shell.which(test_cmd)
            end)
            
            return {
                found_command = success,
                found_path = path or "",
                not_found = not fail_success
            }
        "#;
    
    let result: mlua::Table = lua.load(script).eval().expect("Which command script should succeed");
    
    let found_command: bool = result.get("found_command").unwrap();
    let found_path: String = result.get("found_path").unwrap();
    let not_found: bool = result.get("not_found").unwrap();
    
    // Should find the basic shell command
    assert!(found_command, "Should find basic shell command");
    assert!(!found_path.is_empty(), "Path should not be empty when command is found");
    
    // Should not find the nonexistent command
    assert!(not_found, "Should not find nonexistent command");
}    #[test]
    fn test_lua_shell_platform_specific_workflow() {
        let lua = create_test_lua().unwrap();        let script = r#"
            -- Test a platform-aware CI workflow
            local os_name = shell.os()
            local arch = shell.arch()
            local current_dir = shell.pwd()
            
            -- Build a platform-specific configuration
            local config = {
                platform = os_name .. "-" .. arch,
                working_dir = current_dir,
                package_manager = "unknown"
            }
            
            -- Determine package manager based on OS
            if os_name == "linux" then
                local success, _ = pcall(function() return shell.which("apt-get") end)
                if success then
                    config.package_manager = "apt"
                else
                    local yum_success, _ = pcall(function() return shell.which("yum") end)
                    if yum_success then
                        config.package_manager = "yum"
                    end
                end
            elseif os_name == "macos" then
                local success, _ = pcall(function() return shell.which("brew") end)
                if success then
                    config.package_manager = "brew"
                end
            elseif os_name == "windows" then
                local success, _ = pcall(function() return shell.which("choco") end)
                if success then
                    config.package_manager = "choco"
                else
                    local winget_success, _ = pcall(function() return shell.which("winget") end)
                    if winget_success then
                        config.package_manager = "winget"
                    end
                end
            end
            
            return config
        "#;
    
    let result: mlua::Table = lua.load(script).eval().expect("Platform workflow script should succeed");
    
    let platform: String = result.get("platform").unwrap();
    let working_dir: String = result.get("working_dir").unwrap();
    let package_manager: String = result.get("package_manager").unwrap();
    
    // Verify platform string format
    assert!(platform.contains('-'), "Platform should be in format 'os-arch'");
    
    // Verify working directory
    assert!(!working_dir.is_empty(), "Working directory should not be empty");
    
    // Package manager should be detected or "unknown"
    let known_managers = vec!["apt", "yum", "brew", "choco", "winget", "unknown"];
    assert!(known_managers.contains(&package_manager.as_str()));
}    #[test]
    fn test_lua_shell_environment_variables_integration() {
        let lua = create_test_lua().unwrap();        let script = r#"
            -- Test environment detection with shell execution
            local os_name = shell.os()
            local shell_name = shell.shell()
            
            -- Run a simple command that should work on all platforms
            local test_cmd = "echo test_output"
            if os_name == "windows" then
                test_cmd = "echo test_output"  -- Works on both cmd and PowerShell
            end
            
            local result = shell.run({
                cmd = test_cmd,
                capture = true
            })
            
            return {
                os = os_name,
                shell = shell_name,
                command_success = result.success,
                command_output = result.stdout
            }
        "#;
    
    let result: mlua::Table = lua.load(script).eval().expect("Environment integration script should succeed");
    
    let os: String = result.get("os").unwrap();
    let shell: String = result.get("shell").unwrap();
    let command_success: bool = result.get("command_success").unwrap();
    let command_output: String = result.get("command_output").unwrap();
    
    // Basic validations
    assert!(!os.is_empty(), "OS should be detected");
    assert!(!shell.is_empty(), "Shell should be detected");
    assert!(command_success, "Simple echo command should succeed");
    assert!(command_output.contains("test_output"), "Output should contain test string");
}
