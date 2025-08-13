"""AWS helper utility functions."""

import os
import boto3
from botocore.exceptions import ClientError, NoCredentialsError, ProfileNotFound
from typing import Dict, Optional, Tuple


def get_aws_credentials() -> Optional[Dict[str, str]]:
    """Get AWS credentials from environment variables.
    
    Returns:
        Dictionary with access_key_id, secret_access_key, and region if available,
        None if credentials are not found.
    """
    access_key = os.environ.get('AWS_ACCESS_KEY_ID')
    secret_key = os.environ.get('AWS_SECRET_ACCESS_KEY')
    region = os.environ.get('AWS_DEFAULT_REGION')
    
    if access_key and secret_key:
        return {
            'access_key_id': access_key,
            'secret_access_key': secret_key,
            'region_name': region
        }
    return None


def validate_aws_credentials(access_key: str, secret_key: str, region: str) -> Tuple[bool, str]:
    """Validate AWS credentials by attempting to get caller identity.
    
    Args:
        access_key: AWS access key ID
        secret_key: AWS secret access key
        region: AWS region name
        
    Returns:
        Tuple of (is_valid, message)
    """
    try:
        # Create STS client with provided credentials
        sts_client = boto3.client(
            'sts',
            aws_access_key_id=access_key,
            aws_secret_access_key=secret_key,
            region_name=region
        )
        
        # Test credentials by getting caller identity
        response = sts_client.get_caller_identity()
        
        return True, f"Credentials valid! User: {response.get('Arn', 'Unknown')}"
        
    except NoCredentialsError:
        return False, "No credentials provided"
    except ClientError as e:
        error_code = e.response['Error']['Code']
        if error_code == 'InvalidClientTokenId':
            return False, "Invalid access key ID"
        elif error_code == 'SignatureDoesNotMatch':
            return False, "Invalid secret access key"
        elif error_code == 'UnauthorizedOperation':
            return False, "Insufficient permissions"
        else:
            return False, f"AWS error: {error_code}"
    except Exception as e:
        return False, f"Unexpected error: {str(e)}"


def get_caller_identity(access_key: str, secret_key: str, region: str) -> Tuple[bool, str, Optional[Dict]]:
    """Get AWS caller identity information.
    
    Args:
        access_key: AWS access key ID
        secret_key: AWS secret access key
        region: AWS region name
        
    Returns:
        Tuple of (success, message, identity_data)
    """
    try:
        # Create STS client with provided credentials
        sts_client = boto3.client(
            'sts',
            aws_access_key_id=access_key,
            aws_secret_access_key=secret_key,
            region_name=region
        )
        
        # Get caller identity
        response = sts_client.get_caller_identity()
        
        identity_data = {
            'UserId': response.get('UserId'),
            'Account': response.get('Account'),
            'Arn': response.get('Arn')
        }
        
        return True, "Successfully retrieved caller identity", identity_data
        
    except NoCredentialsError:
        return False, "No credentials provided", None
    except ClientError as e:
        error_code = e.response['Error']['Code']
        if error_code == 'InvalidClientTokenId':
            return False, "Invalid access key ID", None
        elif error_code == 'SignatureDoesNotMatch':
            return False, "Invalid secret access key", None
        elif error_code == 'UnauthorizedOperation':
            return False, "Insufficient permissions", None
        else:
            return False, f"AWS error: {error_code}", None
    except Exception as e:
        return False, f"Unexpected error: {str(e)}", None


def list_available_regions() -> list:
    """Get list of available AWS regions.
    
    Returns:
        List of region names.
    """
    try:
        # Use a default client to get regions
        ec2_client = boto3.client('ec2', region_name='us-east-1')
        regions = ec2_client.describe_regions()
        return [region['RegionName'] for region in regions['Regions']]
    except Exception:
        # Fallback to common regions if API call fails
        return [
            'us-east-1', 'us-east-2', 'us-west-1', 'us-west-2',
            'eu-west-1', 'eu-central-1', 'ap-southeast-1', 'ap-northeast-1'
        ]
