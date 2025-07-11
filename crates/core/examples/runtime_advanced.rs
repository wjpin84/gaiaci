//! # Example: Advanced Pipeline with Dependencies and Conditions
//! 
//! This example demonstrates advanced pipeline features including:
//! - Step dependencies
//! - Conditional execution
//! - Environment variables
//! - File operations
//! - Error handling

use gaiaci_core::lua::runtime::LuaRuntime;
use gaiaci_core::log::{set_logger, info};
use gaiaci_core::log::sinks::stdout::StdoutSink;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging so we can see the Lua log output
    set_logger(StdoutSink);
    info("Starting advanced pipeline example");

    // Create a new runtime
    let mut runtime = LuaRuntime::new()?;

    // Define an advanced pipeline script
    let pipeline_script = r#"
        pipeline = {
            name = "Advanced CI Pipeline",
            description = "A comprehensive example showcasing all features",
            env = {
                PROJECT_NAME = "gaiaci-example",
                BUILD_ENV = "ci",
                ARTIFACT_DIR = "build"
            },
            steps = {
                {
                    name = "setup",
                    description = "Set up the build environment",
                    action = function()
                        gaia.log.info("🔧 Setting up build environment")
                        
                        -- Get environment variables
                        local project = gaia.env.get("PROJECT_NAME")
                        local build_env = gaia.env.get("BUILD_ENV")
                        local artifact_dir = gaia.env.get("ARTIFACT_DIR")
                        
                        gaia.log.info("Project: " .. (project or "unknown"))
                        gaia.log.info("Environment: " .. (build_env or "unknown"))
                        gaia.log.info("Artifact directory: " .. (artifact_dir or "unknown"))
                        
                        -- Create build directory
                        if not gaia.fs.exists(artifact_dir) then
                            gaia.log.info("Creating artifact directory: " .. artifact_dir)
                            gaia.shell.run({cmd = "mkdir -p " .. artifact_dir})
                        else
                            gaia.log.info("Artifact directory already exists")
                        end
                        
                        -- Write a setup marker file
                        gaia.fs.write(artifact_dir .. "/setup.marker", "Setup completed at: " .. os.date())
                        gaia.log.info("✅ Environment setup completed")
                    end
                },
                {
                    name = "build",
                    description = "Build the project",
                    depends_on = {"setup"},
                    action = function()
                        gaia.log.info("🏗️ Building project")
                        
                        -- Check if setup was completed
                        local artifact_dir = gaia.env.get("ARTIFACT_DIR")
                        if not gaia.fs.exists(artifact_dir .. "/setup.marker") then
                            error("Setup step was not completed properly")
                        end
                        
                        -- Simulate build process
                        gaia.log.info("Compiling source code...")
                        local build_result = gaia.shell.run({
                            cmd = "echo 'Simulating build process...' && sleep 1 && echo 'Build completed'",
                            capture = true
                        })
                        
                        if build_result.success then
                            gaia.log.info("Build output: " .. build_result.stdout)
                            
                            -- Create build artifact
                            local project = gaia.env.get("PROJECT_NAME")
                            local artifact_path = artifact_dir .. "/" .. project .. "-binary"
                            gaia.fs.write(artifact_path, "Simulated binary content")
                            gaia.log.info("✅ Build artifact created: " .. artifact_path)
                        else
                            error("Build failed: " .. build_result.stderr)
                        end
                    end
                },
                {
                    name = "test",
                    description = "Run tests",
                    depends_on = {"build"},
                    condition = "gaia.env.get('BUILD_ENV') == 'ci'",
                    action = function()
                        gaia.log.info("🧪 Running tests")
                        
                        -- Check if build was completed
                        local artifact_dir = gaia.env.get("ARTIFACT_DIR")
                        local project = gaia.env.get("PROJECT_NAME")
                        local binary_path = artifact_dir .. "/" .. project .. "-binary"
                        
                        if not gaia.fs.exists(binary_path) then
                            error("Build artifact not found: " .. binary_path)
                        end
                        
                        -- Run tests
                        gaia.log.info("Running unit tests...")
                        local test_result = gaia.shell.run({
                            cmd = "echo 'Running unit tests...' && echo 'All tests passed!'",
                            capture = true
                        })
                        
                        if test_result.success then
                            gaia.log.info("Test output: " .. test_result.stdout)
                            gaia.log.info("✅ All tests passed")
                        else
                            error("Tests failed: " .. test_result.stderr)
                        end
                    end
                },
                {
                    name = "package",
                    description = "Package the build artifacts",
                    depends_on = {"test"},
                    action = function()
                        gaia.log.info("📦 Packaging artifacts")
                        
                        local artifact_dir = gaia.env.get("ARTIFACT_DIR")
                        local project = gaia.env.get("PROJECT_NAME")
                        
                        -- List all files in artifact directory
                        local files = gaia.fs.list_dir(artifact_dir)
                        gaia.log.info("Files in artifact directory:")
                        for i, file in ipairs(files) do
                            gaia.log.info("  - " .. file)
                        end
                        
                        -- Create package
                        local package_name = project .. "-package.tar.gz"
                        local package_cmd = "cd " .. artifact_dir .. " && tar -czf " .. package_name .. " *"
                        
                        local package_result = gaia.shell.run({
                            cmd = package_cmd,
                            capture = true
                        })
                        
                        if package_result.success then
                            gaia.log.info("✅ Package created: " .. package_name)
                        else
                            gaia.log.warn("Package creation failed, but continuing...")
                        end
                    end
                },
                {
                    name = "cleanup",
                    description = "Clean up temporary files",
                    condition = "true", -- Always run cleanup
                    action = function()
                        gaia.log.info("🧹 Cleaning up")
                        
                        local artifact_dir = gaia.env.get("ARTIFACT_DIR")
                        
                        -- Remove temporary files but keep artifacts
                        if gaia.fs.exists(artifact_dir .. "/setup.marker") then
                            gaia.fs.remove(artifact_dir .. "/setup.marker")
                            gaia.log.info("Removed setup marker")
                        end
                        
                        gaia.log.info("✅ Cleanup completed")
                    end
                }
            }
        }
    "#;

    println!("🚀 Loading advanced pipeline...");
    
    // Load the pipeline
    match runtime.load_pipeline(pipeline_script) {
        Ok(pipeline) => {
            println!("✅ Pipeline loaded successfully!");
            println!("   Name: {}", pipeline.name);
            println!("   Description: {}", pipeline.description.as_deref().unwrap_or("None"));
            println!("   Steps: {}", pipeline.steps.len());
            println!("   Environment variables: {}", pipeline.env.len());
            
            // Show step information
            println!("\n📋 Pipeline Steps:");
            for (i, step) in pipeline.steps.iter().enumerate() {
                println!("   {}. {} - {}", i + 1, step.name, 
                    step.description.as_deref().unwrap_or("No description"));
                if !step.depends_on.is_empty() {
                    println!("      Depends on: {:?}", step.depends_on);
                }
                if let Some(condition) = &step.condition {
                    println!("      Condition: {}", condition);
                }
            }
            
            // Execute the pipeline
            println!("\n🔄 Executing pipeline...");
            match runtime.execute_pipeline(&pipeline) {
                Ok(_) => {
                    println!("\n✅ Pipeline executed successfully!");
                    
                    // Show runtime statistics
                    let stats = runtime.stats();
                    println!("\n📊 Runtime Statistics:");
                    println!("   Scripts loaded: {}", stats.scripts_loaded);
                    println!("   Steps executed: {}", stats.steps_executed);
                    println!("   Failed steps: {}", stats.failed_steps);
                    println!("   Total execution time: {}ms", stats.total_execution_time_ms);
                }
                Err(e) => {
                    eprintln!("\n❌ Pipeline execution failed: {}", e);
                    return Err(e.into());
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to load pipeline: {}", e);
            return Err(e.into());
        }
    }

    info("Advanced pipeline example completed");
    Ok(())
}
