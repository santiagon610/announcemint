"""Helper utility functions."""

import os
import sys
from pathlib import Path
from typing import Optional


def get_resource_path(relative_path: str) -> str:
    """Get the absolute path to a resource file.

    Args:
        relative_path: Relative path to the resource.

    Returns:
        Absolute path to the resource.
    """
    if hasattr(sys, "_MEIPASS"):
        # PyInstaller creates a temp folder and stores path in _MEIPASS
        base_path = sys._MEIPASS
    else:
        base_path = os.path.abspath(".")

    return os.path.join(base_path, relative_path)


def ensure_directory(path: str) -> None:
    """Ensure a directory exists, creating it if necessary.

    Args:
        path: Path to the directory.
    """
    Path(path).mkdir(parents=True, exist_ok=True)


def get_app_data_dir() -> str:
    """Get the application data directory.

    Returns:
        Path to the application data directory.
    """
    if sys.platform == "win32":
        app_data = os.environ.get("APPDATA", "")
        return os.path.join(app_data, "Announcemint")
    elif sys.platform == "darwin":
        home = os.path.expanduser("~")
        return os.path.join(home, "Library", "Application Support", "Announcemint")
    else:
        home = os.path.expanduser("~")
        return os.path.join(home, ".config", "announcemint")


def is_windows() -> bool:
    """Check if running on Windows.

    Returns:
        True if running on Windows.
    """
    return sys.platform == "win32"


def is_macos() -> bool:
    """Check if running on macOS.

    Returns:
        True if running on macOS.
    """
    return sys.platform == "darwin"


def is_linux() -> bool:
    """Check if running on Linux.

    Returns:
        True if running on Linux.
    """
    return sys.platform.startswith("linux")
