"""AWS credentials modal dialog."""

import customtkinter as ctk
from typing import Optional, Callable
from announcemint.utils.aws_helpers import (
    validate_aws_credentials,
    get_caller_identity,
    list_available_regions
)


class AWSCredentialsModal:
    """Modal dialog for AWS credentials management."""

    def __init__(self, parent: ctk.CTk, on_credentials_saved: Optional[Callable] = None):
        """Initialize the AWS credentials modal.
        
        Args:
            parent: Parent window
            on_credentials_saved: Callback function when credentials are saved
        """
        self.parent = parent
        self.on_credentials_saved = on_credentials_saved
        self.result = None
        
        # Create modal window
        self.modal = ctk.CTkToplevel(parent)
        self.modal.title("AWS Credentials")
        self.modal.geometry("500x600")
        self.modal.resizable(False, False)
        
        # Make modal modal (block parent interaction)
        self.modal.transient(parent)
        self.modal.grab_set()
        
        # Center modal on parent
        self.center_modal()
        
        self.setup_ui()
        
    def center_modal(self):
        """Center the modal on the parent window."""
        self.parent.update_idletasks()
        parent_x = self.parent.winfo_x()
        parent_y = self.parent.winfo_y()
        parent_width = self.parent.winfo_width()
        parent_height = self.parent.winfo_height()
        
        modal_width = 500
        modal_height = 600
        
        x = parent_x + (parent_width - modal_width) // 2
        y = parent_y + (parent_height - modal_height) // 2
        
        self.modal.geometry(f"{modal_width}x{modal_height}+{x}+{y}")
        
    def setup_ui(self):
        """Set up the user interface."""
        # Main frame
        main_frame = ctk.CTkFrame(self.modal)
        main_frame.pack(fill="both", expand=True, padx=20, pady=20)
        
        # Title
        title_label = ctk.CTkLabel(
            main_frame,
            text="AWS Credentials",
            font=ctk.CTkFont(size=20, weight="bold")
        )
        title_label.pack(pady=(20, 30))
        
        # Credentials form
        self.create_credentials_form(main_frame)
        
        # Buttons
        self.create_buttons(main_frame)
        
        # Status and results area
        self.create_status_area(main_frame)
        
    def create_credentials_form(self, parent):
        """Create the credentials input form."""
        form_frame = ctk.CTkFrame(parent)
        form_frame.pack(fill="x", padx=20, pady=(0, 20))
        
        # Access Key ID
        ctk.CTkLabel(form_frame, text="Access Key ID:").pack(anchor="w", padx=20, pady=(20, 5))
        self.access_key_entry = ctk.CTkEntry(
            form_frame,
            placeholder_text="Enter your AWS Access Key ID",
            width=400
        )
        self.access_key_entry.pack(fill="x", padx=20, pady=(0, 15))
        
        # Secret Access Key
        ctk.CTkLabel(form_frame, text="Secret Access Key:").pack(anchor="w", padx=20, pady=(0, 5))
        self.secret_key_entry = ctk.CTkEntry(
            form_frame,
            placeholder_text="Enter your AWS Secret Access Key",
            show="*",
            width=400
        )
        self.secret_key_entry.pack(fill="x", padx=20, pady=(0, 15))
        
        # Region
        ctk.CTkLabel(form_frame, text="Region:").pack(anchor="w", padx=20, pady=(0, 5))
        
        region_frame = ctk.CTkFrame(form_frame)
        region_frame.pack(fill="x", padx=20, pady=(0, 15))
        
        self.region_var = ctk.StringVar(value="us-east-1")
        self.region_combobox = ctk.CTkComboBox(
            region_frame,
            values=list_available_regions(),
            variable=self.region_var,
            width=300
        )
        self.region_combobox.pack(side="left", padx=(0, 10))
        
        # Show/Hide Secret Key toggle
        self.show_secret_var = ctk.BooleanVar(value=False)
        show_secret_checkbox = ctk.CTkCheckBox(
            region_frame,
            text="Show Secret",
            variable=self.show_secret_var,
            command=self.toggle_secret_visibility
        )
        show_secret_checkbox.pack(side="right")
        
    def create_buttons(self, parent):
        """Create the action buttons."""
        button_frame = ctk.CTkFrame(parent)
        button_frame.pack(fill="x", padx=20, pady=(0, 20))
        
        # Test Credentials button
        self.test_button = ctk.CTkButton(
            button_frame,
            text="Test Credentials",
            command=self.test_credentials,
            width=150
        )
        self.test_button.pack(side="left", padx=(0, 10))
        
        # Get Caller Identity button
        self.identity_button = ctk.CTkButton(
            button_frame,
            text="Get Caller Identity",
            command=self.get_caller_identity,
            width=150
        )
        self.identity_button.pack(side="left", padx=(0, 10))
        
        # Save button
        self.save_button = ctk.CTkButton(
            button_frame,
            text="Save",
            command=self.save_credentials,
            width=100
        )
        self.save_button.pack(side="right")
        
        # Cancel button
        self.cancel_button = ctk.CTkButton(
            button_frame,
            text="Cancel",
            command=self.cancel,
            width=100,
            fg_color="gray"
        )
        self.cancel_button.pack(side="right", padx=(0, 10))
        
    def create_status_area(self, parent):
        """Create the status and results display area."""
        status_frame = ctk.CTkFrame(parent)
        status_frame.pack(fill="both", expand=True, padx=20, pady=(0, 20))
        
        ctk.CTkLabel(
            status_frame,
            text="Status & Results:",
            font=ctk.CTkFont(size=14, weight="bold")
        ).pack(anchor="w", padx=20, pady=(20, 10))
        
        # Status text area
        self.status_text = ctk.CTkTextbox(
            status_frame,
            height=200,
            wrap="word"
        )
        self.status_text.pack(fill="both", expand=True, padx=20, pady=(0, 20))
        
        # Initial status
        self.update_status("Ready to test credentials...")
        
    def toggle_secret_visibility(self):
        """Toggle the visibility of the secret access key."""
        if self.show_secret_var.get():
            self.secret_key_entry.configure(show="")
        else:
            self.secret_key_entry.configure(show="*")
            
    def test_credentials(self):
        """Test the entered AWS credentials."""
        access_key = self.access_key_entry.get().strip()
        secret_key = self.secret_key_entry.get().strip()
        region = self.region_var.get()
        
        if not access_key or not secret_key:
            self.update_status("❌ Please enter both Access Key ID and Secret Access Key")
            return
            
        self.update_status("🔄 Testing credentials...")
        self.test_button.configure(state="disabled")
        
        # Test credentials
        is_valid, message = validate_aws_credentials(access_key, secret_key, region)
        
        if is_valid:
            self.update_status(f"✅ {message}\n\nCredentials are valid and ready to use!")
            self.save_button.configure(state="normal")
        else:
            self.update_status(f"❌ {message}\n\nPlease check your credentials and try again.")
            self.save_button.configure(state="disabled")
            
        self.test_button.configure(state="normal")
        
    def get_caller_identity(self):
        """Get the caller identity using the entered credentials."""
        access_key = self.access_key_entry.get().strip()
        secret_key = self.secret_key_entry.get().strip()
        region = self.region_var.get()
        
        if not access_key or not secret_key:
            self.update_status("❌ Please enter both Access Key ID and Secret Access Key")
            return
            
        self.update_status("🔄 Getting caller identity...")
        self.identity_button.configure(state="disabled")
        
        # Get caller identity
        success, message, identity_data = get_caller_identity(access_key, secret_key, region)
        
        if success and identity_data:
            identity_text = f"✅ {message}\n\n"
            identity_text += f"User ID: {identity_data['UserId']}\n"
            identity_text += f"Account: {identity_data['Account']}\n"
            identity_text += f"ARN: {identity_data['Arn']}"
            self.update_status(identity_text)
        else:
            self.update_status(f"❌ {message}")
            
        self.identity_button.configure(state="normal")
        
    def save_credentials(self):
        """Save the credentials and close the modal."""
        access_key = self.access_key_entry.get().strip()
        secret_key = self.secret_key_entry.get().strip()
        region = self.region_var.get()
        
        if not access_key or not secret_key:
            self.update_status("❌ Please enter both Access Key ID and Secret Access Key")
            return
            
        # Store credentials (in a real app, you might want to encrypt these)
        self.result = {
            'access_key_id': access_key,
            'secret_access_key': secret_key,
            'region_name': region
        }
        
        if self.on_credentials_saved:
            self.on_credentials_saved(self.result)
            
        self.modal.destroy()
        
    def cancel(self):
        """Cancel and close the modal."""
        self.modal.destroy()
        
    def update_status(self, message: str):
        """Update the status display.
        
        Args:
            message: Status message to display
        """
        self.status_text.delete("1.0", "end")
        self.status_text.insert("1.0", message)
        
    def show(self):
        """Show the modal and wait for result.
        
        Returns:
            Dictionary with credentials if saved, None if cancelled
        """
        self.parent.wait_window(self.modal)
        return self.result
