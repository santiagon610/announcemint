#!/bin/bash
# Quick activation script for Announcemint development environment

if [ ! -d "venv" ]; then
    echo "Virtual environment not found. Run ./setup_env.sh first."
    exit 1
fi

echo "Activating Announcemint development environment..."
source venv/bin/activate

echo "Environment activated! Available commands:"
echo "  make test     - Run tests"
echo "  make run      - Run the application"
echo "  make format   - Format code"
echo "  make lint     - Run linting"
echo "  make clean    - Clean build artifacts"
echo ""
echo "To run the application: python -m announcemint.main"
echo "To deactivate: deactivate"
