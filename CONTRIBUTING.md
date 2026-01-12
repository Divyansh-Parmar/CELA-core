# Contributing to CELA

Thank you for your interest in contributing! This project aims to be a robust, safe, and open-source backend for local AI apps.

## Guiding Principles
1.  **Safety First:** The engine must never crash the host application or hang indefinitely.
2.  **Strict Contracts:** The JSON API is our contract. Do not break backward compatibility.
3.  **Minimalism:** We prefer small, verified dependencies over large frameworks.
4.  **Local-Only:** No cloud calls. No telemetry.

## Development Setup
1.  **Prerequisites:**
    - Rust (stable)
    - C/C++ Compiler (clang/gcc)
    - CMake (for llama.cpp)
2.  **Build:**
    ```bash
    cargo build
    ```
3.  **Run Tests:**
    ```bash
    cargo test --workspace
    ```

## Pull Request Process
1.  Fork the repository.
2.  Create a feature branch.
3.  Add tests for your changes.
4.  Ensure `cargo check` and `cargo test` pass.
5.  Submit a PR with a clear description of the change.

## Code Style
- Use `cargo fmt` to format code.
- Use `cargo clippy` to catch common mistakes.
- Document public structs and functions.
