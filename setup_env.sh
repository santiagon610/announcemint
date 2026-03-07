#!/bin/bash

# Announcemint Environment Setup Script
# This script helps users set up and enter the virtual environment

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "pyproject.toml" ]; then
    print_error "Please run this script from the root of the announcemint repository"
    exit 1
fi

print_status "Setting up Announcemint development environment..."

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    print_error "Python 3 is not installed or not in PATH"
    exit 1
fi

# Prefer system Python over Homebrew Python for better tkinter support
if [ -f "/usr/bin/python3" ]; then
    PYTHON_CMD="/usr/bin/python3"
    print_status "Using system Python: $PYTHON_CMD"
elif command -v python3 &> /dev/null; then
    PYTHON_CMD="python3"
    print_status "Using available Python: $PYTHON_CMD"
else
    print_error "No suitable Python 3 found"
    exit 1
fi

# Check Python version
PYTHON_VERSION=$($PYTHON_CMD --version 2>&1 | cut -d' ' -f2 | cut -d'.' -f1,2)
REQUIRED_VERSION="3.10"

if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$PYTHON_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
    print_error "Python 3.10 or higher is required. Found: $PYTHON_VERSION"
    exit 1
fi

print_success "Python version check passed: $($PYTHON_CMD --version)"

# Check if tkinter is available
print_status "Checking for tkinter support..."
if ! $PYTHON_CMD -c "import tkinter" 2>/dev/null; then
    print_warning "tkinter is not available in the system Python"
    print_status "This is required for the GUI application to work"
    print_status "On Ubuntu/Debian, install with: sudo apt-get install python3-tk"
    print_status "On Fedora/RHEL, install with: sudo dnf install python3-tkinter"
    print_status "On macOS with Homebrew, install with: brew install python-tk"
    print_status "On Arch Linux, install with: sudo pacman -S tk"
    echo
    print_warning "Continuing with setup, but GUI may not work until tkinter is installed"
fi

# Remove existing virtual environment if it exists
if [ -d "venv" ]; then
    print_status "Removing existing virtual environment..."
    rm -rf venv
fi

# Create virtual environment
print_status "Creating virtual environment..."
$PYTHON_CMD -m venv venv

# Check if virtual environment was created
if [ ! -f "venv/bin/python" ]; then
    print_error "Failed to create virtual environment"
    exit 1
fi

# Activate virtual environment
print_status "Activating virtual environment..."
source venv/bin/activate

# Check if activation worked
if [ -z "$VIRTUAL_ENV" ]; then
    print_error "Failed to activate virtual environment"
    exit 1
fi

# Upgrade pip
print_status "Upgrading pip..."
python -m pip install --upgrade pip

# Install dependencies
print_status "Installing dependencies..."
python -m pip install customtkinter boto3 Pillow

# Install development dependencies
print_status "Installing development dependencies..."
python -m pip install pytest pytest-cov black flake8 mypy isort

# Install package in development mode
print_status "Installing package in development mode..."
python -m pip install -e .

# Run tests to verify setup
print_status "Running tests to verify setup..."
if python -m pytest tests/ -v; then
    print_success "All tests passed!"
else
    print_warning "Some tests failed, but environment is set up"
fi

# Create activation script
print_status "Creating activation script..."
cat > activate_env.sh << 'EOF'
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
EOF

chmod +x activate_env.sh

print_success "Environment setup complete!"
echo
print_status "To activate the environment in the future, run:"
echo "  source activate_env.sh"
echo "  # or"
echo "  source venv/bin/activate"
echo
print_status "Available commands:"
echo "  make test     - Run tests"
echo "  make run      - Run the application"
echo "  make format   - Format code"
echo "  make lint     - Run linting"
echo "  make clean    - Clean build artifacts"
echo
print_status "To run the application:"
echo "  python -m announcemint.main"
echo
print_warning "Note: If you see tkinter errors, install tkinter support for your system:"
echo "  Ubuntu/Debian: sudo apt-get install python3-tk"
echo "  Fedora/RHEL: sudo dnf install tkinter"
echo "  macOS: brew install python-tk"
echo "  Arch Linux: sudo pacman -S tk"
