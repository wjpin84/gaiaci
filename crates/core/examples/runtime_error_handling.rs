//! # Example: Error Handling and Recovery
//! 
//! This example demonstrates:
//! - Error handling in pipeline steps
//! - Recovery mechanisms
//! - Conditional step execution based on failures

use gaiaci_core::lua::runtime::LuaRuntime;
use gaiaci_core::log::{set_logger, info};
use gaiaci_core::log::sinks::stdout::StdoutSink;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_logger(StdoutSink);
    info("Starting error handling example");

    let mut runtime = LuaRuntime::new()?;

    let pipeline_script = r#"
        pipeline = {
            name = "Error Handling Demo",
            description = "Demonstrates error handling and recovery mechanisms",
            env = {
                SIMULATE_FAILURE = "false",
                RECOVERY_MODE = "true"
            },
            steps = {
                {
                    name = "risky_operation",
                    description = "An operation that might fail",
                    action = function()
                        gaia.log.info("🎲 Running risky operation")
                        
                        local should_fail = gaia.env.get("SIMULATE_FAILURE")
                        if should_fail == "true" then
                            gaia.log.error("💥 Simulated failure occurred!")
                            error("Intentional failure for demonstration")
                        else
                            gaia.log.info("✅ Risky operation completed successfully")
                        end
                    end
                },
                {
                    name = "success_handler",
                    description = "Runs only when risky operation succeeds",
                    depends_on = {"risky_operation"},
                    action = function()
                        gaia.log.info("🎉 Success handler - continuing normal flow")
                    end
                },
                {
                    name = "always_run",
                    description = "This step always runs regardless of failures",
                    condition = "true",
                    action = function()
                        gaia.log.info("🔄 This step always runs")
                        gaia.log.info("Checking system state...")
                        
                        -- This would run cleanup or logging regardless of previous failures
                        gaia.log.info("✅ Always-run step completed")
                    end
                }
            }
        }
    "#;

    // First run with success
    println!("🚀 Running pipeline with success scenario...");
    match runtime.load_pipeline(pipeline_script) {
        Ok(pipeline) => {
            match runtime.execute_pipeline(&pipeline) {
                Ok(_) => {
                    println!("✅ First run completed successfully!");
                    let stats = runtime.stats();
                    println!("   Steps executed: {}, Failed: {}", stats.steps_executed, stats.failed_steps);
                }
                Err(e) => {
                    println!("❌ First run failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to load pipeline: {}", e);
        }
    }

    println!("\n{}", "=".repeat(60));

    // Now test with failure scenario
    let failure_script = r#"
        pipeline = {
            name = "Error Handling Demo - Failure Scenario",
            description = "Demonstrates what happens when steps fail",
            env = {
                SIMULATE_FAILURE = "true",
                RECOVERY_MODE = "true"
            },
            steps = {
                {
                    name = "risky_operation",
                    description = "An operation that will fail",
                    action = function()
                        gaia.log.info("🎲 Running risky operation (will fail)")
                        
                        local should_fail = gaia.env.get("SIMULATE_FAILURE")
                        if should_fail == "true" then
                            gaia.log.error("💥 Simulated failure occurred!")
                            error("Intentional failure for demonstration")
                        else
                            gaia.log.info("✅ Risky operation completed successfully")
                        end
                    end
                },
                {
                    name = "dependent_step",
                    description = "This step depends on risky_operation and won't run if it fails",
                    depends_on = {"risky_operation"},
                    action = function()
                        gaia.log.info("This should not run if risky_operation fails")
                    end
                },
                {
                    name = "error_recovery",
                    description = "Recovery step that runs regardless",
                    condition = "true",
                    action = function()
                        gaia.log.info("🔧 Running error recovery")
                        
                        local recovery_mode = gaia.env.get("RECOVERY_MODE")
                        if recovery_mode == "true" then
                            gaia.log.info("Recovery mode enabled - cleaning up")
                            gaia.log.info("Sending error notifications...")
                            gaia.log.info("Saving failure logs...")
                            gaia.log.info("✅ Recovery completed")
                        end
                    end
                }
            }
        }
    "#;

    println!("\n🚀 Running pipeline with failure scenario...");
    
    // Create a new runtime for the failure test
    let mut failure_runtime = LuaRuntime::new()?;
    
    match failure_runtime.load_pipeline(failure_script) {
        Ok(pipeline) => {
            match failure_runtime.execute_pipeline(&pipeline) {
                Ok(_) => {
                    println!("✅ Second run completed (with partial failures)!");
                }
                Err(e) => {
                    println!("❌ Second run failed: {}", e);
                }
            }
            
            let stats = failure_runtime.stats();
            println!("📊 Failure scenario stats:");
            println!("   Steps executed: {}, Failed: {}", stats.steps_executed, stats.failed_steps);
        }
        Err(e) => {
            println!("❌ Failed to load failure pipeline: {}", e);
        }
    }

    println!("\n🎯 Error handling demonstration completed!");
    println!("This shows how the runtime handles:");
    println!("• Step failures and error propagation");
    println!("• Dependency resolution (failed steps block dependents)");
    println!("• Conditional steps that run regardless of failures");
    println!("• Recovery and cleanup mechanisms");

    info("Error handling example completed");
    Ok(())
}
