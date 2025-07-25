#!/bin/bash

# Setup script for tesseract-rs development environment

echo "Setting up tesseract-rs development environment..."

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js to use commit hooks."
    echo "   Visit: https://nodejs.org/"
    exit 1
fi

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed. Please install npm."
    exit 1
fi

echo "✅ Node.js and npm are installed"

# Install dependencies
echo "Installing Node.js dependencies..."
npm install

# Setup husky
echo "Setting up Git hooks..."
npx husky install

# Make hooks executable
chmod +x .husky/commit-msg

echo "✅ Git hooks installed successfully!"
echo ""
echo "Commit message format:"
echo "  <type>[optional scope]: <description>"
echo ""
echo "Examples:"
echo "  feat: add new OCR feature"
echo "  fix: resolve memory leak issue"
echo "  docs: update installation guide"
echo ""
echo "For more details, see CONTRIBUTING.md"