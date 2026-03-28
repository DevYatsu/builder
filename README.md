# ybuild 🛠️

A zero-config, pragmatic build utility for lazy developers. It detects your project type and runs the right command, then remembers it for next time.

## 🚀 Quick Start

```bash
# Just run it in any project folder
ybuild

# Run and execute (implied by first run in many systems)
ybuild -x

# Run tests
ybuild -t

# Watch for changes and rerun
ybuild -w
```

## ✨ Features

- **Auto-Detection**: Supports 15+ systems including Rust, JavaScript (NPM/Bun/Yarn/PNPM), Python, CMake, Makefile, Just, Go, Docker, Swift, etc.
- **Smart Persistence**: `ybuild` stores your last successful command per directory in `~/.config/ybuild/config.json`. Next time you run `ybuild` in that same folder, it skips detection and runs your previous choice immediately.
- **Granular Selection**:
  - `-s, --system`: If multiple build systems are detected (e.g., both a `Makefile` and `Cargo.toml`), use this to force a re-selection of the underlying system.
  - `-S, --select`: Use this to pick a specific subcommand or target:
    - **NPM/Bun**: Pick a script from `package.json`.
    - **Makefile**: Pick a target (extracted automatically).
    - **CMake**: Pick a build target.
    - **Just**: Pick a recipe.
    - **Python**: Pick a specific script file.
- **Custom Commands**: Use `-c "any shell command"` to override everything. This command will also be remembered for that folder.
- **Watch Mode**: `-w` watches your project files (respecting `.gitignore`) and reruns the last command on every change.

## 📁 Positional Directory Argument

You can run `ybuild` on a specific directory without changing into it:
```bash
ybuild ./my-project -x
```

## ⚙️ Installation

```bash
cargo install --path .
```
