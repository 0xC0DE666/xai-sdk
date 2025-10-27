#!/bin/bash

# xAI Raw Text Sample Example Runner
# This script demonstrates how to run the raw text sampling example

echo "🚀 xAI Raw Text Sampling Example"
echo "================================="
echo ""

# Check if API key is set
if [ -z "$XAI_API_KEY" ]; then
    echo "❌ Error: XAI_API_KEY environment variable is not set"
    echo ""
    echo "Please set your xAI API key:"
    echo "  export XAI_API_KEY=\"your-api-key-here\""
    echo ""
    echo "You can get an API key from: https://console.x.ai"
    exit 1
fi

echo "✅ API key found"
echo "🔧 Building example..."
echo ""

# Build the example
cargo build --example raw_text_sample

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"
echo "🚀 Running example..."
echo ""

# Run the example
cargo run --example raw_text_sample

echo ""
echo "🎉 Example completed!"

