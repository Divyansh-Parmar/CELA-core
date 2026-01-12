#!/bin/bash
set -e

MODEL_URL="https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"
MODEL_PATH="models/default.gguf"

if [ -f "$MODEL_PATH" ]; then
    echo "Model already exists at $MODEL_PATH"
    exit 0
fi

echo "Downloading TinyLlama 1.1B (Q4_K_M)..."
curl -L -o "$MODEL_PATH" "$MODEL_URL"
echo "Download complete."
