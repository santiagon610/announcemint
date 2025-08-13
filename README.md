# Announcemint

A cross-platform Python GUI application built with CustomTkinter.

## Features

- Cross-platform compatibility (Windows, macOS, Linux)
- Modern GUI interface
- Easy to extend and customize

## Requirements

- Python 3.8 or higher
- CustomTkinter (automatically installed with the project)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/announcemint.git
cd announcemint
```

2. Create a virtual environment (recommended):
```bash
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
```

3. Install dependencies:
```bash
pip install -e .
pip install -e ".[dev]"  # For development dependencies
```

## Development Setup

1. Install pre-commit hooks (optional but recommended):
```bash
pre-commit install
```

2. Run linting and formatting:
```bash
# Format code with black
black .

# Sort imports with isort
isort .

# Run type checking with mypy
mypy .

# Run linting with flake8
flake8 .
```

3. Run tests:
```bash
pytest
```

## Running the Application

```bash
python -m announcemint.main
```

## Building Executables

To create standalone executables for distribution:

```bash
# Install PyInstaller
pip install pyinstaller

# Build executable
pyinstaller --onefile --windowed announcemint/main.py
```

## Project Structure

```
announcemint/
├── announcemint/          # Main package
│   ├── __init__.py
│   ├── main.py           # Application entry point
│   ├── gui/              # GUI components
│   │   ├── __init__.py
│   │   ├── main_window.py
│   │   └── widgets.py
│   └── utils/            # Utility functions
│       ├── __init__.py
│       └── helpers.py
├── tests/                 # Test files
├── requirements.txt       # Dependencies
├── pyproject.toml        # Project configuration
└── README.md             # This file
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
