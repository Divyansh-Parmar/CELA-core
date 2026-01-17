# CELA (Computation Engine for Local AI)

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

**A modular, offline-first, production-grade AI runtime engine written in Rust.**

CELA is designed to be the "brain" of your local applications. It wraps high-performance inference backends (currently `llama.cpp`) in a safe, resource-aware, and developer-friendly JSON API.

---

## 🚀 Key Features

*   **Local & Offline:** No cloud dependencies. Your data stays on your machine.
*   **Engine-Owned Memory:** Built-in short-term summary and persistent key-value memory context injected automatically.
*   **Safety First:** Enforces strict limits on tokens, inference time, and context size to prevent crashes or infinite hangs.
*   **JSON Contract:** Predictable input/output schema. No parsing unstructured text to find errors.
*   **Modular:** Designed to support multiple backends (currently optimized for `llama.cpp`).

## ⚠️ What this is NOT

*   **Not a Chatbot UI:** CELA is a backend engine. You build the UI.
*   **Not an Assistant:** It provides capabilities, not personality.
*   **Not a Model Trainer:** This is for inference only.
*   **Not a Vector Database:** The memory is a lightweight, summarized context system, not a RAG pipeline (yet).

---

## 🛠️ Quick Start

### Prerequisites
*   **Linux** (Tested on x86_64, Android cross-compilation experimental).
*   **Rust** (Latest stable).
*   **C/C++ Compiler** & `cmake` (for building underlying llama.cpp).

### 1. Build
```bash
git clone https://github.com/yourusername/cela.git
cd CELA-core
cargo build --release
```

### 2. Download a Model
We provide a helper script to fetch a small test model (TinyLlama 1.1B):
```bash
./scripts/download_model.sh
```
*Alternatively, place any GGUF model in `models/default.gguf`.*

### 3. Run the Server
```bash
./target/release/lie-cli serve
```
*Server listens on `127.0.0.1:8080` by default.*

---

## 🔌 API Usage

### Health Check
```bash
curl http://localhost:8080/v1/health
# {"status":"ok", "version":"1.0.0", ...}
```

### Inference Request
**POST** `/v1/completion`

```json
{
  "prompt": "Explain Rust in one sentence.",
  "limits": {
    "max_tokens": 50,
    "temperature": 0.7
  }
}
```

### Response
```json
{
  "status": "success",
  "intent": null,
  "output": {
    "text": "Rust is a systems programming language that prioritizes safety and performance."
  },
  "usage": {
    "input_tokens": 8,
    "output_tokens": 12,
    "duration_ms": 150
  },
  "error": null
}
```

---

## 🧠 Memory System

CELA features an optional memory layer stored in `memory.json`.

**Enable Memory:**
Start the server or run the CLI with memory enabled (config defaults to off).

**Manage Memory (CLI):**
```bash
./target/release/lie-cli memory set user_name "Alice"
./target/release/lie-cli memory summary "The user is a software engineer."
```

When enabled, these facts are automatically injected into the model's prompt context.

---

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

Apache License 2.0. See [LICENSE](LICENSE) for details.
