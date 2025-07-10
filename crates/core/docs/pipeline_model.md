# GaiaCI Pipeline Model Specification

This document defines the core pipeline structure used in GaiaCI. All pipelines are ultimately converted into this internal model regardless of whether they originate from Lua DSL, JSON, or remote sources.

---

## 📘 Pipeline Overview

A pipeline is a linear or grouped sequence of steps. It is executed in order, with optional support for parallel execution.

```rust
pub struct Pipeline {
    pub steps: Vec<Step>,
    pub env: HashMap<String, String>,
}
```

- `steps`: Ordered list of steps to execute
- `env`: Optional global environment variables

---

## 🔧 Step Definition

```rust
pub enum Step {
    Shell {
        label: String,
        command: String,
    },
    Wasm {
        label: String,
        plugin: String,
        args: Vec<String>,
    },
    Parallel {
        label: String,
        steps: Vec<Step>,
    }
}
```

- `Shell`: Runs a shell command on the runner
- `Wasm`: Executes a WASM plugin by name with args
- `Parallel`: Executes multiple steps concurrently

---

## 🧪 Example (in JSON)

```json
{
  "steps": [
    { "Shell": { "label": "Setup", "command": "echo setup" } },
    {
      "Parallel": {
        "label": "Parallel group",
        "steps": [
          { "Shell": { "label": "Build", "command": "cargo build" } },
          { "Shell": { "label": "Lint", "command": "cargo clippy" } }
        ]
      }
    },
    { "Shell": { "label": "Finish", "command": "echo done" } }
  ],
  "env": {
    "RUST_BACKTRACE": "1"
  }
}
```

---

## 💬 Lua DSL Example

```lua
define_pipeline(function()
  step("Setup", "echo setup")

  parallel("Parallel group", function()
    step("Build", "cargo build")
    step("Lint", "cargo clippy")
  end)

  step("Finish", "echo done")
end)
```

---

## 🔒 Future Extensions

- `Step::InlineLua { label, code }`
- `Step::HttpRequest { method, url, ... }`
- `Step::FileTransfer { from, to }`
- Metadata per step (timeout, retries, tags)

---

## 📌 Invariants

- Step labels must be unique
- Parallel steps are executed concurrently, others sequentially
- Steps are executed in order of appearance unless grouped

---

## 💡 Source Compatibility

- The Lua DSL must produce this model
- Validation and execution operate over this model
- WASM plugins receive step context from this model
