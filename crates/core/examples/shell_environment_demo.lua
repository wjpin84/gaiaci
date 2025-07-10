-- GaiaCI Shell Environment Demo
-- This script demonstrates the new shell environment detection functions

gaia.log.info("=== Shell Environment Detection Demo ===")

-- Basic environment information
local current_dir = gaia.shell.pwd()
local shell_name = gaia.shell.shell()
local os_name = gaia.shell.os()
local arch = gaia.shell.arch()

gaia.log.info("Environment Information:")
gaia.log.info("  Current Directory: " .. current_dir)
gaia.log.info("  Shell: " .. shell_name)
gaia.log.info("  Operating System: " .. os_name)
gaia.log.info("  Architecture: " .. arch)

-- Check for common CI/CD tools
local function check_tool(tool_name)
    local success, path = pcall(function() 
        return gaia.shell.which(tool_name) 
    end)
    
    if success then
        gaia.log.info("✓ " .. tool_name .. " found at: " .. path)
        return true
    else
        gaia.log.warn("✗ " .. tool_name .. " not found in PATH")
        return false
    end
end

gaia.log.info("")
gaia.log.info("=== Tool Availability Check ===")

-- Common development tools
local tools = {
    "git", "node", "npm", "python", "cargo", "docker", 
    "curl", "wget", "make", "cmake"
}

local available_tools = {}
local missing_tools = {}

for _, tool in ipairs(tools) do
    if check_tool(tool) then
        table.insert(available_tools, tool)
    else
        table.insert(missing_tools, tool)
    end
end

gaia.log.info("")
gaia.log.info("=== Platform-Specific CI Commands ===")

-- Demonstrate platform-specific commands
if os_name == "linux" then
    gaia.log.info("Detected Linux - running Linux-specific commands...")
    
    -- Check package manager
    if check_tool("apt-get") then
        gaia.log.info("Using apt package manager")
        -- Example: gaia.shell.run({cmd = "apt-get update", capture = true})
    elseif check_tool("yum") then
        gaia.log.info("Using yum package manager")
    elseif check_tool("pacman") then
        gaia.log.info("Using pacman package manager")
    end
    
elseif os_name == "macos" then
    gaia.log.info("Detected macOS - running macOS-specific commands...")
    
    if check_tool("brew") then
        gaia.log.info("Homebrew is available")
        -- Example: gaia.shell.run({cmd = "brew update", capture = true})
    end
    
elseif os_name == "windows" then
    gaia.log.info("Detected Windows - running Windows-specific commands...")
    
    if check_tool("choco") then
        gaia.log.info("Chocolatey is available")
    end
    
    if check_tool("winget") then
        gaia.log.info("Windows Package Manager is available")
    end
end

-- Architecture-specific logic
gaia.log.info("")
gaia.log.info("=== Architecture-Specific Logic ===")

if arch == "x86_64" then
    gaia.log.info("Running on 64-bit Intel/AMD architecture")
    gaia.log.info("  - Can run most software without compatibility issues")
    gaia.log.info("  - Optimal for high-performance CI builds")
    
elseif arch == "aarch64" then
    gaia.log.info("Running on 64-bit ARM architecture")
    gaia.log.info("  - Modern ARM64 platform (Apple Silicon, ARM servers)")
    gaia.log.info("  - May need ARM-specific binaries for some tools")
    
elseif arch == "x86" then
    gaia.log.info("Running on 32-bit Intel/AMD architecture")
    gaia.log.info("  - Legacy platform, may have memory limitations")
    
else
    gaia.log.info("Running on " .. arch .. " architecture")
    gaia.log.info("  - Specialized platform, may need custom tooling")
end

-- Shell-specific configuration
gaia.log.info("")
gaia.log.info("=== Shell-Specific Configuration ===")

if shell_name == "bash" then
    gaia.log.info("Bash shell detected - can use advanced bash features")
    
elseif shell_name == "zsh" then
    gaia.log.info("Zsh shell detected - can use zsh-specific features")
    
elseif shell_name == "cmd" then
    gaia.log.info("Windows Command Prompt detected")
    
elseif shell_name == "pwsh" or shell_name == "powershell" then
    gaia.log.info("PowerShell detected - can use PowerShell cmdlets")
    
else
    gaia.log.info("Shell: " .. shell_name .. " - using basic POSIX commands")
end

-- Practical CI/CD example: Build matrix detection
gaia.log.info("")
gaia.log.info("=== Build Matrix Detection ===")

local build_config = {
    os = os_name,
    arch = arch,
    shell = shell_name,
    tools = available_tools,
    missing = missing_tools
}

gaia.log.info("Build configuration for this environment:")
gaia.log.info("  Platform: " .. build_config.os .. "-" .. build_config.arch)
gaia.log.info("  Shell: " .. build_config.shell)
gaia.log.info("  Available tools: " .. table.concat(build_config.tools, ", "))

if #build_config.missing > 0 then
    gaia.log.warn("  Missing tools: " .. table.concat(build_config.missing, ", "))
end

-- Suggest installation commands based on platform
if #missing_tools > 0 then
    gaia.log.info("")
    gaia.log.info("=== Tool Installation Suggestions ===")
    
    for _, tool in ipairs(missing_tools) do
        if os_name == "linux" then
            gaia.log.info("  " .. tool .. ": sudo apt-get install " .. tool .. " (or equivalent)")
        elseif os_name == "macos" then
            gaia.log.info("  " .. tool .. ": brew install " .. tool)
        elseif os_name == "windows" then
            gaia.log.info("  " .. tool .. ": choco install " .. tool .. " (or use winget)")
        end
    end
end

gaia.log.info("")
gaia.log.info("=== Demo Complete ===")
gaia.log.info("This environment is ready for CI/CD operations!")
