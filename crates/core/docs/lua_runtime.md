# Gaia Lua Runtime

The Gaia Lua Runtime provides a powerful and flexible system for executing CI/CD pipelines defined in Lua scripts. It offers type-safe pipeline definitions, comprehensive error handling, and seamless integration with Rust.

## Features

- **Type-Safe Pipeline Definitions**: Parse Lua scripts into strongly-typed Rust structs
- **Dependency Resolution**: Automatic step ordering based on dependencies
- **Conditional Execution**: Steps can have conditions for when they should run
- **Environment Variables**: Set and access environment variables from both Rust and Lua
- **Rich Standard Library**: Access to logging, shell commands, file operations, and more
- **Error Handling**: Comprehensive error reporting with stack traces
- **Statistics**: Runtime metrics for monitoring and debugging

## Quick Start

```rust
use gaiaci_core::lua::runtime::LuaRuntime;

// Create a new runtime
let mut runtime = LuaRuntime::new()?;

// Define a pipeline in Lua
let script = r#"
    pipeline = {
        name = "My Pipeline",
        description = "A simple example",
        env = {
            PROJECT_NAME = "my-project"
        },
        steps = {
            {
                name = "build",
                description = "Build the project",
                action = function()
                    gaia.log.info("Building project...")
                    gaia.shell.run({cmd = "cargo build"})
                end
            }
        }
    }
"#;

// Load and execute the pipeline
let pipeline = runtime.load_pipeline(script)?;
runtime.execute_pipeline(&pipeline)?;
```

## Pipeline Structure

A pipeline is defined as a Lua table with the following structure:

```lua
pipeline = {
    name = "Pipeline Name",              -- Required: Human-readable name
    description = "Pipeline description", -- Optional: Description
    env = {                              -- Optional: Environment variables
        VAR_NAME = "value"
    },
    steps = {                            -- Required: Array of steps
        {
            name = "step_name",          -- Required: Unique step identifier
            description = "Step description", -- Optional: Human-readable description
            depends_on = {"other_step"}, -- Optional: Array of step dependencies
            condition = "lua_expression", -- Optional: Condition for execution
            action = function()          -- Required: Lua function to execute
                -- Step implementation
            end
        }
    }
}
```

## Standard Library

The runtime provides a comprehensive standard library accessible via the `gaia` global:

### Logging (`gaia.log`)

```lua
gaia.log.debug("Debug message")
gaia.log.info("Info message")
gaia.log.warn("Warning message")
gaia.log.error("Error message")
```

### Shell Commands (`gaia.shell`)

```lua
-- Simple command execution
gaia.shell.run({cmd = "ls -la"})

-- Capture output
local result = gaia.shell.run({cmd = "git status", capture = true})
if result.success then
    print("Output:", result.stdout)
else
    print("Error:", result.stderr)
end
```

### File System (`gaia.fs`)

```lua
-- Check if file exists
if gaia.fs.exists("Cargo.toml") then
    print("Rust project detected")
end

-- Read file contents
local content = gaia.fs.read("README.md")

-- Write to file
gaia.fs.write("output.txt", "Hello, World!")

-- List directory contents
local files = gaia.fs.list_dir("src")
for i, file in ipairs(files) do
    print("File:", file)
end

-- Remove file
gaia.fs.remove("temp.txt")
```

### Environment Variables (`gaia.env`)

```lua
-- Get environment variable
local home = gaia.env.get("HOME")

-- Set environment variable (for current process)
gaia.env.set("MY_VAR", "my_value")
```

### Assertions (`gaia.assert`)

```lua
-- Basic assertions
gaia.assert.is_true(condition, "Condition must be true")
gaia.assert.equals(actual, expected, "Values must match")
gaia.assert.not_nil(value, "Value cannot be nil")
```

## Step Dependencies

Steps can depend on other steps using the `depends_on` field:

```lua
steps = {
    {
        name = "setup",
        action = function()
            gaia.log.info("Setting up...")
        end
    },
    {
        name = "build",
        depends_on = {"setup"},  -- Will run after setup
        action = function()
            gaia.log.info("Building...")
        end
    },
    {
        name = "test",
        depends_on = {"build"},  -- Will run after build
        action = function()
            gaia.log.info("Testing...")
        end
    }
}
```

## Conditional Execution

Steps can have conditions that determine whether they should run:

```lua
steps = {
    {
        name = "deploy",
        condition = "gaia.env.get('CI') == 'true'",  -- Only run in CI
        action = function()
            gaia.log.info("Deploying...")
        end
    },
    {
        name = "cleanup",
        condition = "true",  -- Always run
        action = function()
            gaia.log.info("Cleaning up...")
        end
    }
}
```

## Error Handling

The runtime provides comprehensive error handling:

1. **Lua Errors**: Runtime errors in Lua code are captured with full stack traces
2. **Step Failures**: Failed steps prevent dependent steps from running
3. **Conditional Steps**: Steps with conditions can run regardless of previous failures
4. **Recovery**: Use conditional steps for cleanup and error recovery

```lua
steps = {
    {
        name = "risky_step",
        action = function()
            if some_condition then
                error("Something went wrong")
            end
        end
    },
    {
        name = "dependent_step",
        depends_on = {"risky_step"},  -- Won't run if risky_step fails
        action = function()
            gaia.log.info("This won't run if risky_step fails")
        end
    },
    {
        name = "cleanup",
        condition = "true",  -- Always runs for cleanup
        action = function()
            gaia.log.info("Cleaning up regardless of failures")
        end
    }
}
```

## Runtime Statistics

Access execution statistics after running a pipeline:

```rust
let stats = runtime.stats();
println!("Scripts loaded: {}", stats.scripts_loaded);
println!("Steps executed: {}", stats.steps_executed);
println!("Failed steps: {}", stats.failed_steps);
println!("Execution time: {}ms", stats.total_execution_time_ms);
```

## Examples

The crate includes several comprehensive examples:

- `runtime_basic.rs` - Simple pipeline execution
- `runtime_advanced.rs` - Advanced features with dependencies and conditions
- `runtime_error_handling.rs` - Error handling and recovery scenarios

Run examples with:

```bash
cargo run --example runtime_basic
cargo run --example runtime_advanced
cargo run --example runtime_error_handling
```

## Integration with CI Systems

The runtime is designed to integrate seamlessly with various CI systems:

### GitHub Actions

```yaml
- name: Run Gaia Pipeline
  run: |
    echo 'pipeline = { ... }' > pipeline.lua
    cargo run --bin gaia-runner pipeline.lua
```

### GitLab CI

```yaml
script:
  - cargo run --bin gaia-runner .gitlab/pipeline.lua
```

### Custom Integration

```rust
use gaiaci_core::lua::runtime::LuaRuntime;

fn run_ci_pipeline(script_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = LuaRuntime::new()?;
    let script = std::fs::read_to_string(script_path)?;
    let pipeline = runtime.load_pipeline(&script)?;
    runtime.execute_pipeline(&pipeline)?;
    Ok(())
}
```

## Best Practices

1. **Use Descriptive Names**: Give steps and pipelines meaningful names and descriptions
2. **Handle Errors Gracefully**: Use conditional steps for cleanup and error recovery
3. **Leverage Dependencies**: Use step dependencies to ensure proper execution order
4. **Environment Variables**: Use environment variables for configuration
5. **Logging**: Use appropriate log levels for better debugging
6. **Modular Steps**: Keep steps focused on single responsibilities
7. **Testing**: Test your pipelines with different scenarios and conditions
