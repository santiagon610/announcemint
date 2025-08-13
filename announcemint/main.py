#!/usr/bin/env python3
"""Main entry point for the Announcemint application."""

import sys
from tkinter import messagebox
from typing import Optional

import customtkinter as ctk

from announcemint.gui.main_window import MainWindow


class Application:
    """Main application class."""

    def __init__(self) -> None:
        """Initialize the application."""
        self.root: Optional[ctk.CTk] = None
        self.main_window: Optional[MainWindow] = None

    def setup(self) -> None:
        """Set up the main application window."""
        # Configure CustomTkinter appearance
        ctk.set_appearance_mode("dark")  # Options: "dark", "light", "system"
        ctk.set_default_color_theme("blue")  # Options: "blue", "green", "dark-blue"

        self.root = ctk.CTk()
        self.root.title("Announcemint")
        self.root.geometry("900x700")

        # Set window icon if available
        try:
            # You can add an icon file here
            # self.root.iconbitmap("path/to/icon.ico")
            pass
        except Exception:
            pass

        # Configure window properties
        self.root.protocol("WM_DELETE_WINDOW", self.on_closing)

        # Create main window
        self.main_window = MainWindow(self.root)

    def run(self) -> None:
        """Run the application main loop."""
        if not self.root:
            raise RuntimeError("Application not properly initialized")

        try:
            self.root.mainloop()
        except KeyboardInterrupt:
            self.cleanup()
        except Exception as e:
            messagebox.showerror("Error", f"An unexpected error occurred: {e}")
            self.cleanup()

    def on_closing(self) -> None:
        """Handle application closing."""
        if messagebox.askokcancel("Quit", "Do you want to quit?"):
            self.cleanup()
            if self.root:
                self.root.destroy()

    def cleanup(self) -> None:
        """Clean up resources before exit."""
        # Add any cleanup code here
        pass


def main() -> None:
    """Main function to start the application."""
    try:
        app = Application()
        app.setup()
        app.run()
    except Exception as e:
        print(f"Failed to start application: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
