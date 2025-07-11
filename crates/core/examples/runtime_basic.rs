//! # Example: Basic Lua Runtime Usage
//! 
//! This example demonstrates how to use the GaiaCI Lua runtime to load
//! and execute pipeline scripts.

use gaiaci_core::lua::runtime::{LuaRuntime, RuntimeError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new runtime
    let mut runtime = LuaRuntime::new()?;

    // Define a simple pipeline script
    let pipeline_script = r#"
        pipeline = {
            name = "Hello World Pipeline",
            description = "A simple example pipeline",
            env = {
                MESSAGE = "Hello from GaiaCI!"
            },
            steps = {
                {
                    name = "greeting",
                    description = "Print a greeting message",
                    action = function()
                        local msg = gaia.env.get("MESSAGE") or "Hello World!"
                        gaia.log.info("Step message: " .. msg)
                        gaia.log.info("Pipeline step completed successfully")
                    end
                },
                {
                    name = "system_check",
                    description = "Check system information",
                    action = function()
                        gaia.log.info("Current directory: " .. gaia.shell.pwd())
                        gaia.log.info("Operating system: " .. gaia.shell.os())
                        gaia.log.info("Architecture: " .. gaia.shell.arch())
                        
                        -- Test shell command execution
                        local result = gaia.shell.run({
                            cmd = "echo 'Hello from shell!'",
                            capture = true
                        })
                        
                        if result.success then
                            gaia.log.info("Shell output: " .. result.stdout)
                        else
                            gaia.log.error("Shell command failed")
                        end
                    end
                }
            }
        }
    "#;

    println!("🚀 Loading pipeline...");
    
    // Load the pipeline
    match runtime.load_pipeline(pipeline_script) {
        Ok(pipeline) => {
            println!("✅ Pipeline loaded successfully!");
            println!("   Name: {}", pipeline.name);
            println!("   Steps: {}", pipeline.steps.len());
            
            // Execute the pipeline
            println!("🔄 Executing pipeline...");
            match runtime.execute_pipeline(&pipeline) {
                Ok(_) => {
                    println!("✅ Pipeline executed successfully!");
                    
                    // Show runtime statistics
                    let stats = runtime.stats();
                    println!("📊 Runtime Statistics:");
                    println!("   Scripts loaded: {}", stats.scripts_loaded);
                    println!("   Steps executed: {}", stats.steps_executed);
                    println!("   Failed steps: {}", stats.failed_steps);
                    println!("   Total execution time: {}ms", stats.total_execution_time_ms);
                }
                Err(e) => {
                    eprintln!("❌ Pipeline execution failed: {}", e);
                    return Err(e.into());
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to load pipeline: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
