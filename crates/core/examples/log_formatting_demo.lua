-- GaiaCI Log Formatting Demo
-- This script demonstrates the configurable log formatting capabilities

gaia.log.info("=== GaiaCI Log Formatting Demo ===")
gaia.log.info("Starting demonstration of configurable log formats...")

-- 1. Default/Simple Format
gaia.log.info("1. Using default simple format:")
gaia.log.info("This is an info message")
gaia.log.warn("This is a warning message")
gaia.log.error("This is an error message")

-- 2. CI/CD Format
gaia.log.info("\n2. Switching to CI/CD format:")
gaia.log.set_ci_format()
gaia.log.info("Build process started")
gaia.log.warn("Using deprecated configuration option")
gaia.log.info("Tests are running...")
gaia.log.info("Build completed successfully")

-- 3. Development Format
gaia.log.info("\n3. Switching to development format:")
gaia.log.set_dev_format()
gaia.log.debug("Debug message with thread info")
gaia.log.trace("Trace message for detailed debugging")
gaia.log.info("Development build in progress")

-- 4. JSON Format
gaia.log.info("\n4. Switching to JSON format:")
gaia.log.set_json_format()
gaia.log.info("JSON formatted message for log aggregation")
gaia.log.warn("JSON warning with structured data")
gaia.log.error("JSON error message")

-- 5. Custom Template Format
gaia.log.set_simple_format()  -- Switch back to read the next message clearly
gaia.log.info("\n5. Using custom template format:")
gaia.log.set_format("🕐 {timestamp:%H:%M:%S} | {level:lower} | {message}")
gaia.log.info("Custom formatted info message")
gaia.log.warn("Custom formatted warning")

-- 6. Another Custom Format (CI Pipeline Style)
gaia.log.set_simple_format()
gaia.log.info("\n6. CI Pipeline style format:")
gaia.log.set_format("[{timestamp:%Y-%m-%d %H:%M:%S}] 🚀 [{level}] {message}")
gaia.log.info("Pipeline stage: Checkout")
gaia.log.info("Pipeline stage: Build")
gaia.log.warn("Pipeline stage: Test (some tests skipped)")
gaia.log.info("Pipeline stage: Deploy")

-- 7. Detailed Format with Process Info
gaia.log.set_simple_format()
gaia.log.info("\n7. Detailed format with process information:")
gaia.log.set_format("[{timestamp:%H:%M:%S}] [PID:{pid}] [{level}] {message}")
gaia.log.info("Detailed process tracking")
gaia.log.debug("Memory usage within acceptable limits")
gaia.log.info("All operations completed")

-- Reset to simple format
gaia.log.set_simple_format()
gaia.log.info("\n=== Demo completed! ===")
gaia.log.info("Log formatting allows for:")
gaia.log.info("✓ Flexible timestamp formats")
gaia.log.info("✓ Multiple preset formats (CI, dev, JSON)")
gaia.log.info("✓ Custom template placeholders")
gaia.log.info("✓ Process and thread information")
gaia.log.info("✓ Easy switching between formats")
