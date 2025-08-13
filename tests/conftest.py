"""Test configuration and fixtures."""

import sys
from unittest.mock import MagicMock

# Mock customtkinter before any tests import it
mock_ctk = MagicMock()
mock_ctk.CTk = MagicMock()
mock_ctk.CTkFrame = MagicMock()
mock_ctk.CTkLabel = MagicMock()
mock_ctk.CTkEntry = MagicMock()
mock_ctk.CTkButton = MagicMock()
mock_ctk.CTkTextbox = MagicMock()
mock_ctk.CTkFont = MagicMock()
mock_ctk.StringVar = MagicMock()
mock_ctk.set_appearance_mode = MagicMock()
mock_ctk.set_default_color_theme = MagicMock()

# Configure the CTk mock to return a title when title() is called
mock_ctk_instance = MagicMock()
mock_ctk_instance.title.return_value = "Announcemint"
mock_ctk_instance.geometry = MagicMock()
mock_ctk_instance.protocol = MagicMock()
mock_ctk_instance.grid_rowconfigure = MagicMock()
mock_ctk_instance.grid_columnconfigure = MagicMock()
mock_ctk_instance.grid = MagicMock()
mock_ctk_instance.pack = MagicMock()
mock_ctk_instance.destroy = MagicMock()
mock_ctk.CTk.return_value = mock_ctk_instance

# Mock tkinter components
mock_tkinter = MagicMock()
mock_tkinter.messagebox = MagicMock()
mock_tkinter.messagebox.askokcancel = MagicMock()
mock_tkinter.messagebox.showerror = MagicMock()
mock_tkinter.messagebox.showwarning = MagicMock()

# Replace the modules in sys.modules
sys.modules['customtkinter'] = mock_ctk
sys.modules['tkinter'] = mock_tkinter
sys.modules['tkinter.messagebox'] = mock_tkinter.messagebox
