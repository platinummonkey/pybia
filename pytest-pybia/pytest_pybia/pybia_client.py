"""
Client for interacting with the PyBia tool.
"""

import os
import subprocess
import json
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple, Any


class PyBiaClient:
    """
    Client for interacting with the PyBia tool.
    """

    def __init__(self, root_dir: Path):
        self.root_dir = root_dir
        self.pybia_path = self._find_pybia_executable()

    def _find_pybia_executable(self) -> str:
        """
        Find the PyBia executable in the PATH.
        """
        # Try to find pybia in PATH
        try:
            result = subprocess.run(
                ["which", "pybia"], 
                capture_output=True, 
                text=True, 
                check=True
            )
            return result.stdout.strip()
        except subprocess.CalledProcessError:
            # Fall back to assuming it's in the PATH
            return "pybia"

    def get_impacted_services(
        self, 
        config_file: Optional[str] = None,
        changed_files: Optional[List[str]] = None,
        base_commit: Optional[str] = None
    ) -> List[Tuple[str, str]]:
        """
        Get services impacted by changes.
        
        Args:
            config_file: Path to PyBia configuration file
            changed_files: List of changed files to analyze
            base_commit: Git base commit to compare against
            
        Returns:
            List of tuples (service_name, service_path)
        """
        cmd = [self.pybia_path, "--paths", str(self.root_dir), "--service-format", "name-path"]
        
        if config_file:
            cmd.extend(["--services-config", config_file])
            
        # Handle changed files
        if changed_files:
            # Write changed files to a temporary file
            temp_file = self.root_dir / ".pybia_changed_files.txt"
            with open(temp_file, "w") as f:
                f.write("\n".join(changed_files))
            
            cmd.extend(["--changed-files", str(temp_file)])
        elif base_commit:
            # Get changed files from git
            git_cmd = ["git", "diff", "--name-only", base_commit]
            try:
                git_files = subprocess.run(
                    git_cmd, 
                    capture_output=True, 
                    text=True, 
                    check=True
                ).stdout.strip().split("\n")
                
                # Filter out empty lines
                git_files = [f for f in git_files if f]
                
                if git_files:
                    # Write changed files to a temporary file
                    temp_file = self.root_dir / ".pybia_changed_files.txt"
                    with open(temp_file, "w") as f:
                        f.write("\n".join(git_files))
                    
                    cmd.extend(["--changed-files", str(temp_file)])
            except subprocess.CalledProcessError as e:
                print(f"Warning: Failed to get changed files from git: {e}")
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            
            # Parse the output
            impacted_services = []
            for line in result.stdout.strip().split("\n"):
                if not line:
                    continue
                    
                parts = line.split(",")
                if len(parts) >= 2:
                    service_name = parts[0]
                    service_path = parts[1]
                    impacted_services.append((service_name, service_path))
            
            # Clean up temporary file if it exists
            temp_file = self.root_dir / ".pybia_changed_files.txt"
            if temp_file.exists():
                temp_file.unlink()
                
            return impacted_services
                
        except subprocess.CalledProcessError as e:
            print(f"Error running PyBia: {e}")
            print(f"stdout: {e.stdout}")
            print(f"stderr: {e.stderr}")
            
            # Clean up temporary file if it exists
            temp_file = self.root_dir / ".pybia_changed_files.txt"
            if temp_file.exists():
                temp_file.unlink()
                
            return []

    def get_changed_files(self, base_commit: str) -> List[str]:
        """
        Get files changed since the base commit.
        
        Args:
            base_commit: Git base commit to compare against
            
        Returns:
            List of changed file paths
        """
        git_cmd = ["git", "diff", "--name-only", base_commit]
        try:
            result = subprocess.run(
                git_cmd, 
                capture_output=True, 
                text=True, 
                check=True
            )
            
            # Filter out empty lines and non-Python files
            changed_files = [
                f for f in result.stdout.strip().split("\n") 
                if f and (f.endswith(".py") or f.endswith(".toml") or f.endswith(".txt"))
            ]
            
            # Convert to absolute paths
            return [os.path.join(str(self.root_dir), f) for f in changed_files]
        except subprocess.CalledProcessError as e:
            print(f"Warning: Failed to get changed files from git: {e}")
            return []

    def get_service_files(self, service_path: str) -> List[str]:
        """
        Get all Python files in a service.
        
        Args:
            service_path: Path to the service
            
        Returns:
            List of Python file paths
        """
        files = []
        for root, _, filenames in os.walk(service_path):
            for filename in filenames:
                if filename.endswith(".py"):
                    files.append(os.path.join(root, filename))
        return files 