"""Tests for utility functions."""

import os
import sys
from unittest.mock import MagicMock, patch

import pytest

from announcemint.utils.helpers import (
    ensure_directory,
    get_app_data_dir,
    get_resource_path,
    is_linux,
    is_macos,
    is_windows,
)


class TestGetResourcePath:
    """Test cases for get_resource_path function."""

    def test_normal_path(self) -> None:
        """Test getting resource path in normal mode."""
        result = get_resource_path("test.txt")
        expected = os.path.join(os.path.abspath("."), "test.txt")
        assert result == expected

    def test_pyinstaller_path(self) -> None:
        """Test getting resource path in PyInstaller mode."""
        # Mock the sys._MEIPASS attribute by temporarily adding it
        original_sys = sys.__dict__.copy()
        sys._MEIPASS = "/tmp/pyinstaller"  # type: ignore
        try:
            result = get_resource_path("test.txt")
            expected = "/tmp/pyinstaller/test.txt"
            # Normalize paths to handle different path separator representations
            assert os.path.normpath(result) == os.path.normpath(expected)
        finally:
            # Restore original sys state
            sys.__dict__.clear()
            sys.__dict__.update(original_sys)


class TestEnsureDirectory:
    """Test cases for ensure_directory function."""

    def test_create_new_directory(self, tmp_path: str) -> None:
        """Test creating a new directory."""
        new_dir = os.path.join(tmp_path, "new_dir")
        ensure_directory(new_dir)
        assert os.path.exists(new_dir)
        assert os.path.isdir(new_dir)

    def test_existing_directory(self, tmp_path: str) -> None:
        """Test with existing directory."""
        ensure_directory(tmp_path)
        assert os.path.exists(tmp_path)

    def test_nested_directory(self, tmp_path: str) -> None:
        """Test creating nested directories."""
        nested_dir = os.path.join(tmp_path, "parent", "child", "grandchild")
        ensure_directory(nested_dir)
        assert os.path.exists(nested_dir)
        assert os.path.isdir(nested_dir)


class TestGetAppDataDir:
    """Test cases for get_app_data_dir function."""

    @patch("sys.platform", "win32")
    @patch.dict(os.environ, {"APPDATA": "C:\\Users\\Test\\AppData\\Roaming"})
    def test_windows_path(self) -> None:
        """Test getting app data directory on Windows."""
        result = get_app_data_dir()
        # Use os.path.join to handle path separators correctly
        expected = os.path.join("C:\\Users\\Test\\AppData\\Roaming", "Announcemint")
        # Normalize paths to handle different path separator representations
        assert os.path.normpath(result) == os.path.normpath(expected)

    @patch("sys.platform", "darwin")
    @patch("os.path.expanduser")
    def test_macos_path(self, mock_expanduser: MagicMock) -> None:
        """Test getting app data directory on macOS."""
        mock_expanduser.return_value = "/Users/test"
        result = get_app_data_dir()
        expected = "/Users/test/Library/Application Support/Announcemint"
        # Normalize paths to handle different path separator representations
        assert os.path.normpath(result) == os.path.normpath(expected)

    @patch("sys.platform", "linux")
    @patch("os.path.expanduser")
    def test_linux_path(self, mock_expanduser: MagicMock) -> None:
        """Test getting app data directory on Linux."""
        mock_expanduser.return_value = "/home/test"
        result = get_app_data_dir()
        expected = "/home/test/.config/announcemint"
        # Normalize paths to handle different path separator representations
        assert os.path.normpath(result) == os.path.normpath(expected)


class TestPlatformDetection:
    """Test cases for platform detection functions."""

    @patch("sys.platform", "win32")
    def test_is_windows(self) -> None:
        """Test Windows detection."""
        assert is_windows() is True
        assert is_macos() is False
        assert is_linux() is False

    @patch("sys.platform", "darwin")
    def test_is_macos(self) -> None:
        """Test macOS detection."""
        assert is_windows() is False
        assert is_macos() is True
        assert is_linux() is False

    @patch("sys.platform", "linux")
    def test_is_linux(self) -> None:
        """Test Linux detection."""
        assert is_windows() is False
        assert is_macos() is False
        assert is_linux() is True

    @patch("sys.platform", "linux-x86_64")
    def test_is_linux_variant(self) -> None:
        """Test Linux variant detection."""
        assert is_linux() is True
