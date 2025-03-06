"""
Impact analysis module for pytest-pybia.

This module combines PyBia results with dependency analysis to determine
which tests are impacted by code changes.
"""

import os
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple

from .dependency import DependencyAnalyzer
from .pybia_client import PyBiaClient


class ImpactAnalyzer:
    """
    Analyzes the impact of code changes on tests.
    """

    def __init__(self, root_dir: Path):
        self.root_dir = root_dir
        self.dependency_analyzer = DependencyAnalyzer(root_dir)
        self.pybia_client = PyBiaClient(root_dir)

        # Results
        self.impacted_services: List[Tuple[str, str]] = []
        self.impacted_files: Set[str] = set()
        self.impacted_modules: Set[str] = set()
        self.changed_files: List[str] = []

        # Mapping of modules to services
        self.module_to_service: Dict[str, str] = {}

    def scan_codebase(self) -> None:
        """
        Scan the codebase to build the dependency graph.
        """
        self.dependency_analyzer.scan_directory()

    def analyze_impact(
        self,
        config_file: Optional[str] = None,
        changed_files: Optional[List[str]] = None,
        base_commit: Optional[str] = None,
    ) -> None:
        """
        Analyze the impact of code changes.

        Args:
            config_file: Path to PyBia configuration file
            changed_files: List of changed files to analyze
            base_commit: Git base commit to compare against
        """
        # Get changed files if not provided
        if not changed_files and base_commit:
            changed_files = self.pybia_client.get_changed_files(base_commit)

        self.changed_files = changed_files or []

        # Get impacted services from PyBia
        self.impacted_services = self.pybia_client.get_impacted_services(
            config_file=config_file,
            changed_files=changed_files,
            base_commit=base_commit,
        )

        # Get all files in impacted services
        for service_name, service_path in self.impacted_services:
            service_files = self.pybia_client.get_service_files(service_path)
            for file in service_files:
                self.impacted_files.add(file)

                # Map module to service
                module_name = self._file_to_module(file)
                if module_name:
                    self.impacted_modules.add(module_name)
                    self.module_to_service[module_name] = service_name

        # Add transitive dependencies
        if changed_files:
            # Get files impacted by transitive dependencies
            transitive_files = self.dependency_analyzer.get_impacted_files(
                changed_files
            )
            self.impacted_files.update(transitive_files)

            # Update impacted modules
            for file in transitive_files:
                module_name = self._file_to_module(file)
                if module_name:
                    self.impacted_modules.add(module_name)

    def is_file_impacted(self, file_path: str) -> bool:
        """
        Check if a file is impacted by the changes.
        """
        # Check direct file impact
        if file_path in self.impacted_files:
            return True

        # Check module
        module_name = self._file_to_module(file_path)
        return module_name and module_name in self.impacted_modules

    def is_module_impacted(self, module_name: str) -> bool:
        """
        Check if a module is impacted by changes.

        Args:
            module_name: Name of the module

        Returns:
            True if the module is impacted, False otherwise
        """
        # Direct check
        if module_name in self.impacted_modules:
            return True

        # Check if module is in an impacted service
        if module_name in self.module_to_service:
            service_name = self.module_to_service[module_name]
            if any(s[0] == service_name for s in self.impacted_services):
                return True

        return False

    def _file_to_module(self, file_path: str) -> Optional[str]:
        """
        Convert a file path to a Python module name.
        """
        try:
            # Get relative path from project root
            rel_path = os.path.relpath(file_path, str(self.root_dir))

            # Convert path to module
            return rel_path.replace(".py", "").replace(os.path.sep, ".")
        except Exception:
            return None

    def get_impact_summary(self) -> Dict[str, Any]:
        """
        Get a summary of the impact analysis.

        Returns:
            Dictionary with impact summary information
        """
        # Count files per service
        service_file_counts = {}
        for service_name, service_path in self.impacted_services:
            service_files = [
                f for f in self.impacted_files if f.startswith(service_path)
            ]
            service_file_counts[service_name] = len(service_files)

        return {
            "changed_files": self.changed_files,
            "impacted_services": self.impacted_services,
            "service_file_counts": service_file_counts,
            "impacted_files_count": len(self.impacted_files),
            "impacted_modules_count": len(self.impacted_modules),
        }
