# GaiaCI Log Formatting System

The GaiaCI log formatting system provides flexible, configurable log output formatting for Lua scripts. This allows you to customize log messages to fit different environments, use cases, and integration requirements.

## Overview

The logging system supports:
- **Template-based formatting** with placeholder substitution
- **Preset formats** for common use cases (CI/CD, development, JSON)
- **Custom formatters** for specialized requirements
- **Dynamic format switching** during script execution
- **Rich context information** (timestamps, process info, user data)

## Available Functions

### Basic Logging Functions
```lua
log.trace(message)   -- Most verbose level
log.debug(message)   -- Debug information
log.info(message)    -- General information
log.warn(message)    -- Warning messages
log.error(message)   -- Error messages
```

### Format Configuration Functions
```lua
log.set_format(template)     -- Set custom template
log.set_ci_format()         -- CI/CD friendly format
log.set_json_format()       -- JSON structured output
log.set_dev_format()        -- Development/debugging format
log.set_simple_format()     -- Basic format (default)
```

## Format Templates

### Template Placeholders

| Placeholder | Description | Example |
|-------------|-------------|---------|
| `{timestamp}` | Full ISO timestamp | `2023-07-10T14:30:15Z` |
| `{timestamp:format}` | Custom timestamp format | `{timestamp:%H:%M:%S}` → `14:30:15` |
| `{level}` | Log level uppercase | `INFO`, `WARN`, `ERROR` |
| `{level:lower}` | Log level lowercase | `info`, `warn`, `error` |
| `{message}` | The log message content | User-provided message |
| `{user}` | User identifier | `ci-bot`, `developer`, `unknown` |
| `{pid}` | Process ID | `12345` |
| `{thread}` | Thread identifier | `ThreadId(1)` |
| `{source}` | Source location | `file.rs:42` |
| `{field:name}` | Custom field value | User-defined fields |

### Timestamp Formatting

The `{timestamp:format}` placeholder uses [chrono formatting patterns](https://docs.rs/chrono/latest/chrono/format/strftime/index.html):

```lua
-- Common timestamp formats
log.set_format("[{timestamp:%Y-%m-%d %H:%M:%S}] {message}")     -- 2023-07-10 14:30:15
log.set_format("[{timestamp:%H:%M:%S}] {message}")             -- 14:30:15
log.set_format("[{timestamp:%b %d %H:%M}] {message}")          -- Jul 10 14:30
log.set_format("[{timestamp:%Y%m%d_%H%M%S}] {message}")        -- 20230710_143015
```

## Preset Formats

### 1. Simple Format (Default)
```lua
log.set_simple_format()
log.info("Application started")
-- Output: [INFO] Application started
```

### 2. CI/CD Format
```lua
log.set_ci_format()
log.info("Build completed")
-- Output: [2023-07-10 14:30:15] [unknown] [INFO] Build completed
```

### 3. JSON Format
```lua
log.set_json_format()
log.info("Process completed")
-- Output: {"timestamp":"2023-07-10T14:30:15Z","level":"info","message":"Process completed","user":"unknown","pid":12345}
```

### 4. Development Format
```lua
log.set_dev_format()
log.debug("Debug information")
-- Output: [14:30:15] [ThreadId(1)] [DEBUG] Debug information
```

## Custom Template Examples

### Basic Custom Format
```lua
log.set_format("[{level}] {message}")
log.info("Custom format")
-- Output: [INFO] Custom format
```

### Timestamped Format
```lua
log.set_format("{timestamp:%H:%M:%S} | {level:lower} | {message}")
log.warn("Warning message")
-- Output: 14:30:15 | warn | Warning message
```

### CI Pipeline Format
```lua
log.set_format("🚀 [{timestamp:%Y-%m-%d %H:%M:%S}] [{level}] {message}")
log.info("Deploy started")
-- Output: 🚀 [2023-07-10 14:30:15] [INFO] Deploy started
```

### Detailed Debug Format
```lua
log.set_format("[{timestamp:%H:%M:%S}] [PID:{pid}] [Thread:{thread}] [{level}] {message}")
log.debug("Detailed debug info")
-- Output: [14:30:15] [PID:12345] [Thread:ThreadId(1)] [DEBUG] Detailed debug info
```

### Compact Format
```lua
log.set_format("{timestamp:%H%M%S} {level:lower}: {message}")
log.error("Compact error")
-- Output: 143015 error: Compact error
```

## Real-World Examples

### CI/CD Pipeline Logging
```lua
-- Configure for CI environment
log.set_ci_format()

-- Pipeline execution
log.info("=== CI Pipeline Started ===")
log.info("Stage: Checkout")
log.info("Repository: https://github.com/user/repo")

log.info("Stage: Build") 
log.warn("Using cached dependencies")
log.info("Build completed successfully")

log.info("Stage: Test")
log.info("Running 127 tests...")
log.info("All tests passed")

log.info("Stage: Deploy")
log.info("Deploying to production")
log.info("=== Pipeline Completed ===")
```

### Development Environment
```lua
-- Configure for development
log.set_dev_format()

-- Development logging
log.trace("Function entry: process_config()")
log.debug("Config file loaded: /etc/app.conf")
log.debug("Processing 45 configuration items")
log.info("Configuration validation completed")
log.trace("Function exit: process_config()")
```

### JSON Logging for Log Aggregation
```lua
-- Configure for structured logging
log.set_json_format()

-- Structured logs that can be easily parsed
log.info("User authentication successful")
log.warn("Rate limit approaching for API key")
log.error("Database connection failed")
log.info("Request processed successfully")
```

### Custom Business Logic Format
```lua
-- Custom format for business metrics
log.set_format("📊 {timestamp:%H:%M:%S} | {level} | {message}")

log.info("Daily report generation started")
log.info("Processing 1,234 transactions")
log.warn("High CPU usage detected: 85%")
log.info("Report generation completed")
```

## Dynamic Format Switching

You can change formats during script execution:

```lua
-- Start with simple format
log.set_simple_format()
log.info("Application initialization")

-- Switch to detailed format for debugging
log.set_dev_format()
log.debug("Detailed debug information")
log.trace("Very verbose trace data")

-- Switch to JSON for structured data
log.set_json_format()
log.info("Structured log entry")

-- Back to simple for final status
log.set_simple_format()
log.info("Application ready")
```

## Best Practices

### 1. Choose Appropriate Formats by Environment
```lua
-- In CI/CD environments
log.set_ci_format()

-- In development
log.set_dev_format()

-- For log aggregation systems
log.set_json_format()
```

### 2. Use Consistent Formatting Within Phases
```lua
-- Configure once at the beginning of a phase
log.set_format("[{timestamp:%H:%M:%S}] BUILD | {level} | {message}")

-- Use consistently throughout the phase
log.info("Starting build process")
log.info("Compiling source files")
log.warn("Deprecated API usage detected")
log.info("Build completed successfully")
```

### 3. Include Context Information
```lua
-- Include relevant context in custom formats
log.set_format("[{timestamp:%Y-%m-%d %H:%M:%S}] [Build:{field:build_id}] [{level}] {message}")
```

### 4. Consider Log Volume and Performance
```lua
-- For high-volume logging, prefer simpler formats
log.set_simple_format()

-- For detailed analysis, use rich formats
log.set_format("[{timestamp}] [PID:{pid}] [Thread:{thread}] [{level}] {message}")
```

## Migration from Basic Logging

The formatting system is backward compatible. Existing code continues to work:

```lua
-- This still works exactly as before
log.info("Application started")
log.warn("Configuration missing")
log.error("Connection failed")

-- Add formatting when needed
log.set_ci_format()
log.info("Now using CI format")
```

## Integration with External Systems

### ELK Stack (Elasticsearch, Logstash, Kibana)
```lua
-- Use JSON format for easy parsing
log.set_json_format()
log.info("Event processed")
```

### Splunk
```lua
-- Use structured format with clear delimiters
log.set_format("timestamp={timestamp} level={level} message=\"{message}\"")
```

### CloudWatch / DataDog
```lua
-- Include relevant metadata
log.set_format("[{timestamp}] [{level}] [service=gaiaci] {message}")
```

This flexible formatting system makes GaiaCI logs suitable for any environment while maintaining ease of use and powerful customization capabilities!
