//! # Pipeline Parser
//! 
//! This module handles parsing Lua tables into Pipeline and PipelineStep structs,
//! with comprehensive error handling and validation.

use mlua::{Table, Function};
use std::collections::HashMap;
use crate::lua::pipeline::{PipelineError, PipelineConfig, RetryConfig};
use crate::lua::pipeline::step::{Pipeline, PipelineStep};

/// Pipeline parser responsible for converting Lua tables to Pipeline structs
pub struct PipelineParser;

impl PipelineParser {
    /// Parses a pipeline table from Lua into a Pipeline struct
    pub fn parse_pipeline_table(table: Table) -> Result<Pipeline, PipelineError> {
        let name: String = table.get("name")
            .map_err(|_| PipelineError::ParseError("Pipeline must have a 'name' field".to_string()))?;
        
        let description: Option<String> = table.get("description").ok();
        let version: Option<String> = table.get("version").ok();
        let env: HashMap<String, String> = table.get("env").unwrap_or_default();
        let metadata: HashMap<String, String> = table.get("metadata").unwrap_or_default();
        
        // Parse configuration if present
        let config = if let Ok(config_table) = table.get::<Table>("config") {
            Self::parse_config_table(config_table)?
        } else {
            PipelineConfig::default()
        };
        
        // Parse steps manually
        let steps_table: Table = table.get("steps")
            .map_err(|_| PipelineError::ParseError("Pipeline must have a 'steps' field".to_string()))?;
        
        let mut steps = Vec::new();
        for i in 1..=steps_table.len()? {
            let step_table: Table = steps_table.get(i)?;
            let step = Self::parse_step_table(step_table, i as usize)?;
            steps.push(step);
        }
        
        Ok(Pipeline {
            name,
            description,
            env,
            steps,
            config,
            metadata,
            version,
        })
    }
    
    /// Parses a single step table
    pub fn parse_step_table(table: Table, index: usize) -> Result<PipelineStep, PipelineError> {
        let name: String = table.get("name")
            .map_err(|_| PipelineError::ParseError(format!("Step {} must have a 'name' field", index)))?;
        
        let description: Option<String> = table.get("description").ok();
        let condition: Option<String> = table.get("condition").ok();
        let env: HashMap<String, String> = table.get("env").unwrap_or_default();
        let parallel: bool = table.get("parallel").unwrap_or(false);
        let depends_on: Vec<String> = table.get("depends_on").unwrap_or_default();
        let critical: bool = table.get("critical").unwrap_or(false);
        let timeout_seconds: Option<u64> = table.get("timeout").ok();
        let working_directory: Option<String> = table.get("working_directory").ok();
        let metadata: HashMap<String, String> = table.get("metadata").unwrap_or_default();
        
        // Parse retry configuration if present
        let retry = if let Ok(retry_table) = table.get::<Table>("retry") {
            Self::parse_retry_table(retry_table)?
        } else {
            RetryConfig::default()
        };
        
        // Store action as an execution reference
        let action = if table.get::<Function>("action").is_ok() {
            Some(format!("pipeline.steps[{}].action()", index))
        } else {
            None
        };
        
        Ok(PipelineStep {
            name,
            description,
            condition,
            env,
            parallel,
            depends_on,
            action,
            retry,
            timeout_seconds,
            critical,
            metadata,
            working_directory,
        })
    }
    
    /// Parses a configuration table
    pub fn parse_config_table(table: Table) -> Result<PipelineConfig, PipelineError> {
        let timeout_seconds: u64 = table.get("timeout_seconds").unwrap_or(3600);
        let continue_on_error: bool = table.get("continue_on_error").unwrap_or(false);
        let working_directory: Option<String> = table.get("working_directory").ok();
        let global_env: HashMap<String, String> = table.get("global_env").unwrap_or_default();
        let max_memory_bytes: Option<u64> = table.get("max_memory_bytes").ok();
        let verbose: bool = table.get("verbose").unwrap_or(false);
        
        // Parse parallel configuration if present
        let parallel = if let Ok(parallel_table) = table.get::<Table>("parallel") {
            Self::parse_parallel_table(parallel_table)?
        } else {
            crate::lua::pipeline::ParallelConfig::default()
        };
        
        Ok(PipelineConfig {
            timeout_seconds,
            continue_on_error,
            working_directory,
            parallel,
            global_env,
            max_memory_bytes,
            verbose,
        })
    }
    
    /// Parses a parallel configuration table
    pub fn parse_parallel_table(table: Table) -> Result<crate::lua::pipeline::ParallelConfig, PipelineError> {
        let enabled: bool = table.get("enabled").unwrap_or(false);
        let max_concurrent: usize = table.get("max_concurrent").unwrap_or(4);
        let dependency_timeout_seconds: u64 = table.get("dependency_timeout_seconds").unwrap_or(300);
        
        // Parse failure strategy
        let failure_strategy = if let Ok(strategy_str) = table.get::<String>("failure_strategy") {
            Self::parse_failure_strategy(&strategy_str)?
        } else {
            crate::lua::pipeline::config::FailureStrategy::default()
        };
        
        Ok(crate::lua::pipeline::ParallelConfig {
            enabled,
            max_concurrent,
            dependency_timeout_seconds,
            failure_strategy,
        })
    }
    
    /// Parses a retry configuration table
    pub fn parse_retry_table(table: Table) -> Result<RetryConfig, PipelineError> {
        let max_attempts: u32 = table.get("max_attempts").unwrap_or(0);
        let delay_seconds: u64 = table.get("delay_seconds").unwrap_or(1);
        let exponential_backoff: bool = table.get("exponential_backoff").unwrap_or(false);
        let max_delay_seconds: u64 = table.get("max_delay_seconds").unwrap_or(60);
        
        // Parse retry conditions if present
        let retry_conditions = if let Ok(conditions_table) = table.get::<Table>("conditions") {
            Self::parse_retry_conditions(conditions_table)?
        } else {
            vec![crate::lua::pipeline::config::RetryCondition::AnyFailure]
        };
        
        Ok(RetryConfig {
            max_attempts,
            delay_seconds,
            exponential_backoff,
            max_delay_seconds,
            retry_conditions,
        })
    }
    
    /// Parses retry conditions from a table
    pub fn parse_retry_conditions(table: Table) -> Result<Vec<crate::lua::pipeline::config::RetryCondition>, PipelineError> {
        let mut conditions = Vec::new();
        
        for i in 1..=table.len()? {
            if let Ok(condition_str) = table.get::<String>(i) {
                let condition = Self::parse_retry_condition(&condition_str)?;
                conditions.push(condition);
            } else if let Ok(condition_table) = table.get::<Table>(i) {
                let condition = Self::parse_retry_condition_table(condition_table)?;
                conditions.push(condition);
            }
        }
        
        if conditions.is_empty() {
            conditions.push(crate::lua::pipeline::config::RetryCondition::AnyFailure);
        }
        
        Ok(conditions)
    }
    
    /// Parses a single retry condition from string
    pub fn parse_retry_condition(condition_str: &str) -> Result<crate::lua::pipeline::config::RetryCondition, PipelineError> {
        use crate::lua::pipeline::config::RetryCondition;
        
        match condition_str.to_lowercase().as_str() {
            "any" | "any_failure" => Ok(RetryCondition::AnyFailure),
            "timeout" => Ok(RetryCondition::Timeout),
            _ => {
                if condition_str.starts_with("exit_code:") {
                    let code_str = condition_str.strip_prefix("exit_code:").unwrap();
                    let code: i32 = code_str.parse()
                        .map_err(|_| PipelineError::ParseError(format!("Invalid exit code: {}", code_str)))?;
                    Ok(RetryCondition::ExitCodes(vec![code]))
                } else if condition_str.starts_with("error:") {
                    let pattern = condition_str.strip_prefix("error:").unwrap().to_string();
                    Ok(RetryCondition::ErrorPattern(pattern))
                } else {
                    // Treat as Lua condition
                    Ok(RetryCondition::LuaCondition(condition_str.to_string()))
                }
            }
        }
    }
    
    /// Parses a retry condition from a table
    pub fn parse_retry_condition_table(table: Table) -> Result<crate::lua::pipeline::config::RetryCondition, PipelineError> {
        use crate::lua::pipeline::config::RetryCondition;
        
        if let Ok(exit_codes) = table.get::<Vec<i32>>("exit_codes") {
            Ok(RetryCondition::ExitCodes(exit_codes))
        } else if let Ok(pattern) = table.get::<String>("error_pattern") {
            Ok(RetryCondition::ErrorPattern(pattern))
        } else if let Ok(lua_condition) = table.get::<String>("lua_condition") {
            Ok(RetryCondition::LuaCondition(lua_condition))
        } else {
            Err(PipelineError::ParseError("Invalid retry condition table".to_string()))
        }
    }
    
    /// Parses a failure strategy from string
    pub fn parse_failure_strategy(strategy_str: &str) -> Result<crate::lua::pipeline::config::FailureStrategy, PipelineError> {
        use crate::lua::pipeline::config::FailureStrategy;
        
        match strategy_str.to_lowercase().as_str() {
            "fail_fast" => Ok(FailureStrategy::FailFast),
            "continue_on_failure" => Ok(FailureStrategy::ContinueOnFailure),
            "critical_only" => Ok(FailureStrategy::CriticalOnly),
            _ => {
                // Treat as custom Lua strategy
                Ok(FailureStrategy::Custom(strategy_str.to_string()))
            }
        }
    }
    
    /// Validates a parsed pipeline for common issues
    pub fn validate_parsed_pipeline(pipeline: &Pipeline) -> Result<(), PipelineError> {
        // Check for empty pipeline
        if pipeline.steps.is_empty() {
            return Err(PipelineError::ParseError("Pipeline must have at least one step".to_string()));
        }
        
        // Check for duplicate step names
        let mut step_names = std::collections::HashSet::new();
        for step in &pipeline.steps {
            if !step_names.insert(&step.name) {
                return Err(PipelineError::ParseError(format!("Duplicate step name: '{}'", step.name)));
            }
        }
        
        // Check that all dependencies reference existing steps
        for step in &pipeline.steps {
            for dep in &step.depends_on {
                if !step_names.contains(dep) {
                    return Err(PipelineError::DependencyError(
                        format!("Step '{}' depends on non-existent step '{}'", step.name, dep)
                    ));
                }
            }
        }
        
        // Check for steps with no action
        for step in &pipeline.steps {
            if step.action.is_none() {
                return Err(PipelineError::ParseError(
                    format!("Step '{}' has no action defined", step.name)
                ));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;

    fn create_test_lua() -> Lua {
        Lua::new()
    }

    #[test]
    fn test_parse_failure_strategy() {
        assert!(matches!(
            PipelineParser::parse_failure_strategy("fail_fast").unwrap(),
            crate::lua::pipeline::config::FailureStrategy::FailFast
        ));
        
        assert!(matches!(
            PipelineParser::parse_failure_strategy("continue_on_failure").unwrap(),
            crate::lua::pipeline::config::FailureStrategy::ContinueOnFailure
        ));
        
        if let crate::lua::pipeline::config::FailureStrategy::Custom(custom) = 
            PipelineParser::parse_failure_strategy("custom_strategy").unwrap() {
            assert_eq!(custom, "custom_strategy");
        } else {
            panic!("Expected custom failure strategy");
        }
    }

    #[test]
    fn test_parse_retry_condition() {
        use crate::lua::pipeline::config::RetryCondition;
        
        assert!(matches!(
            PipelineParser::parse_retry_condition("any_failure").unwrap(),
            RetryCondition::AnyFailure
        ));
        
        assert!(matches!(
            PipelineParser::parse_retry_condition("timeout").unwrap(),
            RetryCondition::Timeout
        ));
        
        if let RetryCondition::ExitCodes(codes) = 
            PipelineParser::parse_retry_condition("exit_code:1").unwrap() {
            assert_eq!(codes, vec![1]);
        } else {
            panic!("Expected exit code condition");
        }
        
        if let RetryCondition::ErrorPattern(pattern) = 
            PipelineParser::parse_retry_condition("error:connection failed").unwrap() {
            assert_eq!(pattern, "connection failed");
        } else {
            panic!("Expected error pattern condition");
        }
    }

    #[test]
    fn test_validate_parsed_pipeline() {
        // Valid pipeline
        let mut valid_pipeline = Pipeline::new("test");
        valid_pipeline.add_step(PipelineStep::new("step1").with_action("action1"));
        assert!(PipelineParser::validate_parsed_pipeline(&valid_pipeline).is_ok());
        
        // Empty pipeline
        let empty_pipeline = Pipeline::new("empty");
        assert!(PipelineParser::validate_parsed_pipeline(&empty_pipeline).is_err());
        
        // Duplicate step names
        let mut duplicate_pipeline = Pipeline::new("dup");
        duplicate_pipeline.add_step(PipelineStep::new("step1").with_action("action1"));
        duplicate_pipeline.add_step(PipelineStep::new("step1").with_action("action2"));
        assert!(PipelineParser::validate_parsed_pipeline(&duplicate_pipeline).is_err());
        
        // Invalid dependency
        let mut invalid_dep_pipeline = Pipeline::new("invalid");
        invalid_dep_pipeline.add_step(PipelineStep::new("step1")
            .with_action("action1")
            .with_dependency("nonexistent"));
        assert!(PipelineParser::validate_parsed_pipeline(&invalid_dep_pipeline).is_err());
    }

    #[test]
    fn test_parse_simple_pipeline_table() {
        let lua = create_test_lua();
        let pipeline_script = r#"
            return {
                name = "test-pipeline",
                description = "A test pipeline",
                steps = {
                    {
                        name = "step1",
                        description = "First step",
                        action = function() end
                    }
                }
            }
        "#;
        
        let table: Table = lua.load(pipeline_script).eval().unwrap();
        let pipeline = PipelineParser::parse_pipeline_table(table).unwrap();
        
        assert_eq!(pipeline.name, "test-pipeline");
        assert_eq!(pipeline.description, Some("A test pipeline".to_string()));
        assert_eq!(pipeline.steps.len(), 1);
        assert_eq!(pipeline.steps[0].name, "step1");
    }
}
