#!/bin/bash
# Script to format all Python code with Ruff

# Install Ruff if not already installed
pip install -U ruff

# Format all Python files
echo "Formatting Python files with Ruff..."
ruff format .

# Run linting and auto-fix what can be fixed
echo "Running Ruff linting with auto-fix..."
ruff check --fix .

echo "Done!" 