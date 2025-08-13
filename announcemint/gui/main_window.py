"""Main window GUI implementation."""

from tkinter import messagebox
from typing import Any

import customtkinter as ctk
from announcemint.utils.aws_helpers import get_aws_credentials
from announcemint.gui.aws_modal import AWSCredentialsModal


class MainWindow:
    """Main application window."""

    def __init__(self, parent: ctk.CTk) -> None:
        """Initialize the main window.

        Args:
            parent: The parent CustomTkinter widget.
        """
        self.parent = parent
        self.aws_credentials = None
        self.setup_ui()
        self.check_aws_credentials()

    def setup_ui(self) -> None:
        """Set up the user interface."""
        # Configure grid weights for responsive layout
        self.parent.grid_rowconfigure(0, weight=1)
        self.parent.grid_columnconfigure(0, weight=1)

        # Create main frame
        self.main_frame = ctk.CTkFrame(self.parent)
        self.main_frame.grid(row=0, column=0, sticky="nsew", padx=20, pady=20)

        # Configure main frame grid
        self.main_frame.grid_columnconfigure(0, weight=1)
        self.main_frame.grid_columnconfigure(1, weight=1)

        self.create_header()
        self.create_content()
        self.create_footer()

    def create_header(self) -> None:
        """Create the header section."""
        header_frame = ctk.CTkFrame(self.main_frame)
        header_frame.grid(row=0, column=0, columnspan=2, sticky="ew", pady=(0, 20))

        # Title
        title_label = ctk.CTkLabel(
            header_frame, text="Announcemint", font=ctk.CTkFont(size=24, weight="bold")
        )
        title_label.pack(pady=10)

        # Subtitle
        subtitle_label = ctk.CTkLabel(
            header_frame,
            text="Cross-platform Python GUI Application",
            font=ctk.CTkFont(size=14),
            text_color="gray",
        )
        subtitle_label.pack(pady=(0, 10))

    def create_content(self) -> None:
        """Create the main content section."""
        # Left column
        left_frame = ctk.CTkFrame(self.main_frame)
        left_frame.grid(row=1, column=0, sticky="nsew", padx=(0, 10))

        # Input label
        input_label = ctk.CTkLabel(
            left_frame, text="Input", font=ctk.CTkFont(size=16, weight="bold")
        )
        input_label.pack(anchor="w", padx=20, pady=(20, 10))

        # Input field
        ctk.CTkLabel(left_frame, text="Enter text:").pack(
            anchor="w", padx=20, pady=(0, 5)
        )
        self.input_entry = ctk.CTkEntry(
            left_frame, placeholder_text="Type something here...", width=300
        )
        self.input_entry.pack(fill="x", padx=20, pady=(0, 20))

        # Buttons
        button_frame = ctk.CTkFrame(left_frame)
        button_frame.pack(fill="x", padx=20, pady=(0, 20))

        self.submit_btn = ctk.CTkButton(
            button_frame, text="Submit", command=self.on_submit
        )
        self.submit_btn.pack(side="left", padx=(0, 10))

        self.clear_btn = ctk.CTkButton(
            button_frame, text="Clear", command=self.on_clear
        )
        self.clear_btn.pack(side="left")

        # AWS Section
        self.create_aws_section(left_frame)

        # Right column
        right_frame = ctk.CTkFrame(self.main_frame)
        right_frame.grid(row=1, column=1, sticky="nsew", padx=(10, 0))

        # Output label
        output_label = ctk.CTkLabel(
            right_frame, text="Output", font=ctk.CTkFont(size=16, weight="bold")
        )
        output_label.pack(anchor="w", padx=20, pady=(20, 10))

        # Output text area
        ctk.CTkLabel(right_frame, text="Results:").pack(
            anchor="w", padx=20, pady=(0, 5)
        )

        self.output_text = ctk.CTkTextbox(right_frame, height=300, width=300)
        self.output_text.pack(fill="both", expand=True, padx=20, pady=(0, 20))

    def create_footer(self) -> None:
        """Create the footer section."""
        footer_frame = ctk.CTkFrame(self.main_frame)
        footer_frame.grid(row=2, column=0, columnspan=2, sticky="ew", pady=(20, 0))

        # Status bar
        self.status_var = ctk.StringVar(value="Ready")
        status_label = ctk.CTkLabel(
            footer_frame,
            textvariable=self.status_var,
            font=ctk.CTkFont(size=12),
            text_color="gray",
        )
        status_label.pack(side="left", padx=20, pady=10)

        # Version info
        version_label = ctk.CTkLabel(
            footer_frame, text="v0.1.0", font=ctk.CTkFont(size=12), text_color="gray"
        )
        version_label.pack(side="right", padx=20, pady=10)

    def on_submit(self) -> None:
        """Handle submit button click."""
        text = self.input_entry.get()
        if not text.strip():
            messagebox.showwarning("Warning", "Please enter some text first.")
            return

        # Process the input (you can customize this)
        result = f"Processed: {text.upper()}"

        # Update output
        self.output_text.insert("end", result + "\n")
        self.output_text.see("end")

        # Update status
        self.status_var.set(f"Processed: {len(text)} characters")

        # Clear input
        self.input_entry.delete(0, "end")

    def on_clear(self) -> None:
        """Handle clear button click."""
        self.input_entry.delete(0, "end")
        self.output_text.delete("1.0", "end")
        self.status_var.set("Ready")

    def create_aws_section(self, parent) -> None:
        """Create the AWS credentials section."""
        # AWS section frame
        aws_frame = ctk.CTkFrame(parent)
        aws_frame.pack(fill="x", padx=20, pady=(20, 0))

        # AWS section title
        aws_title = ctk.CTkLabel(
            aws_frame, text="AWS Integration", font=ctk.CTkFont(size=16, weight="bold")
        )
        aws_title.pack(anchor="w", padx=20, pady=(20, 15))

        # AWS status
        self.aws_status_label = ctk.CTkLabel(
            aws_frame, text="Checking credentials...", text_color="gray"
        )
        self.aws_status_label.pack(anchor="w", padx=20, pady=(0, 15))

        # AWS buttons frame
        aws_button_frame = ctk.CTkFrame(aws_frame)
        aws_button_frame.pack(fill="x", padx=20, pady=(0, 20))

        # Check credentials button
        self.check_aws_btn = ctk.CTkButton(
            aws_button_frame, text="Check Credentials", command=self.check_aws_credentials
        )
        self.check_aws_btn.pack(side="left", padx=(0, 10))

        # Manage credentials button
        self.manage_aws_btn = ctk.CTkButton(
            aws_button_frame, text="Manage Credentials", command=self.manage_aws_credentials
        )
        self.manage_aws_btn.pack(side="left")

    def check_aws_credentials(self) -> None:
        """Check for existing AWS credentials in environment."""
        self.aws_credentials = get_aws_credentials()
        
        if self.aws_credentials:
            self.aws_status_label.configure(
                text=f"✅ Credentials found for region: {self.aws_credentials['region_name']}",
                text_color="green"
            )
            self.check_aws_btn.configure(text="Refresh Credentials")
        else:
            self.aws_status_label.configure(
                text="❌ No AWS credentials found in environment",
                text_color="red"
            )
            self.check_aws_btn.configure(text="Check Credentials")

    def manage_aws_credentials(self) -> None:
        """Open the AWS credentials management modal."""
        modal = AWSCredentialsModal(self.parent, self.on_aws_credentials_saved)
        result = modal.show()
        
        if result:
            self.aws_credentials = result
            self.aws_status_label.configure(
                text=f"✅ Credentials saved for region: {result['region_name']}",
                text_color="green"
            )
            self.check_aws_btn.configure(text="Refresh Credentials")
            
            # Update output with success message
            self.output_text.insert("end", f"AWS credentials saved successfully for region: {result['region_name']}\n")
            self.output_text.see("end")

    def on_aws_credentials_saved(self, credentials) -> None:
        """Callback when AWS credentials are saved."""
        # This method is called by the modal when credentials are saved
        pass
