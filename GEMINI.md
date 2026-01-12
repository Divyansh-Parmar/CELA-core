# CELA - Context & Status

## Project Vision
To provide a production-grade, open-source-ready, local AI engine that serves as a shared intelligence layer for applications. It is not a chatbot; it is a backend engine that manages models, memory, and task routing, exposing capabilities via a structured API.

## Core Design Philosophy
1.  **Engine, not Assistant:** The system provides capabilities, not personality.
2.  **Strictly Local:** Offline-first design.
3.  **Modular & Agnostic:** The core engine knows nothing about specific model architectures; it uses adapters.
4.  **Resource Aware:** Respects memory limits and enforces constraints.
5.  **JSON-First:** All communication is structured.

## Engine Contract (Locked)
*(See README.md for public contract)*

## Architecture

```mermaid
graph TD
    Client[Client App] --> |HTTP/JSON| Server[Service Wrapper (Axum)]
    Client --> |Rust Lib| Core[Core Engine Crate]
    
    subgraph "CELA"
        Server --> Core
        
        Core --> Router[Task Router]
        Core --> Memory[Memory Manager]
        Core --> Runtime[Runtime Adapter Interface]
        
        Memory --> |Read/Write| JSON[memory.json]
        
        Runtime --> |Trait Impl| LlamaCpp[llama.cpp Adapter]
    end
    
    LlamaCpp --> |FFI| LibLlama[libllama.so/.dll]
```

## Status: Release Ready (v1.0.0)

### Completed Phases
1.  **Architecture & Core:** Implemented modular Rust core.
2.  **Runtime Integration:** Integrated `llama.cpp` via bindings.
3.  **Contract Locking:** Enforced strict JSON API and resource limits.
4.  **Memory Layer:** Implemented engine-owned persistent memory.
5.  **Service Hardening:** Validated inputs, concurrency, and error handling.
6.  **Android Spike:** Confirmed feasibility (requires patching deps).
7.  **Reference App:** Verified usage with `lie-ref-client`.
8.  **Open-Source Prep:** Cleaned up repo, added License, CoC, Contributing guidelines.

### Open-Source Readiness Checklist
- [x] **Audit:** No sensitive files committed.
- [x] **Ignore:** `.gitignore` covers models and logs.
- [x] **Docs:** README is user-facing. CONTRIBUTING/SECURITY/CODE_OF_CONDUCT added.
- [x] **License:** Apache 2.0 applied.
- [x] **Build:** `cargo build` verifies cleanly.

### Public Positioning
"CELA is the robust, safe, and modular backend for the next generation of offline-first AI applications. It handles the hard parts—memory, limits, and runtime management—so developers can focus on building features."

## Next Steps (Post-Release)
- **CI/CD:** Set up GitHub Actions for automated builds.
- **Android Port:** Apply patches to `llama-cpp-sys-2` and release Android libs.
- **Memory v2:** Implement semantic search/vector store integration.