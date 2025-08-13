.PHONY: help install install-dev test lint format clean build run

help:  ## Show this help message
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

install:  ## Install the package
	pip install -e .

install-dev:  ## Install development dependencies
	pip install -e ".[dev]"
	pre-commit install

test:  ## Run tests
	pytest

test-cov:  ## Run tests with coverage
	pytest --cov=announcemint --cov-report=html --cov-report=term-missing

lint:  ## Run linting checks
	flake8 .
	mypy .

format:  ## Format code
	black .
	isort .

format-check:  ## Check code formatting
	black --check --diff .
	isort --check-only --diff .

clean:  ## Clean up build artifacts
	rm -rf build/
	rm -rf dist/
	rm -rf *.egg-info/
	rm -rf .pytest_cache/
	rm -rf .mypy_cache/
	rm -rf htmlcov/
	find . -type f -name "*.pyc" -delete
	find . -type d -name "__pycache__" -delete

build:  ## Build the package
	python -m build

run:  ## Run the application
	python -m announcemint.main

check-all: format-check lint test  ## Run all checks
