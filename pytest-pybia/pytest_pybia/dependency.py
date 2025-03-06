"""
Dependency analyzer for pytest-pybia.

This module analyzes Python files to determine their dependencies.
"""

import ast
import os
from pathlib import Path
from typing import Dict, List, Optional, Set


class DependencyAnalyzer:
    """
    Analyzes Python files to determine their dependencies.
    """

    def __init__(self, root_dir: Path):
        self.root_dir = root_dir
        self.module_dependencies: Dict[str, Set[str]] = {}
        self.file_dependencies: Dict[str, Set[str]] = {}
        self.module_to_file: Dict[str, str] = {}
        self.file_to_module: Dict[str, str] = {}

    def scan_directory(self, directory: Path = None) -> None:
        """
        Scan a directory for Python files and analyze their dependencies.
        """
        if directory is None:
            directory = self.root_dir

        for root, _, files in os.walk(directory):
            for file in files:
                if file.endswith(".py"):
                    file_path = os.path.join(root, file)
                    self.analyze_file(file_path)

    def analyze_file(self, file_path: str) -> None:
        """
        Analyze a Python file to determine its imports and dependencies.
        """
        try:
            with open(file_path, encoding="utf-8") as f:
                content = f.read()

            module_name = self._file_to_module(file_path)
            if not module_name:
                return

            self.module_to_file[module_name] = file_path
            self.file_to_module[file_path] = module_name

            # Parse the file and extract imports
            tree = ast.parse(content)
            imports = self._extract_imports(tree)

            # Map imports to modules
            dependencies = set()
            for imp in imports:
                # Handle relative imports
                if imp.startswith("."):
                    # Relative import
                    parent_module = self._get_parent_module(module_name)
                    imp = imp[1:]  # Remove the leading dot

                full_import = f"{parent_module}.{imp}" if imp else parent_module

                dependencies.add(full_import)

            self.module_dependencies[module_name] = dependencies
            self.file_dependencies[file_path] = set()

            # Map dependencies to files
            for dep in dependencies:
                # Find the file for this module
                dep_file = self._find_module_file(dep)
                if dep_file:
                    self.file_dependencies[file_path].add(dep_file)

        except Exception as e:
            print(f"Error analyzing {file_path}: {e}")

    def _extract_imports(self, tree: ast.AST) -> Set[str]:
        """
        Extract imports from an AST.
        """
        imports = set()

        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for name in node.names:
                    imports.add(name.name)
            elif isinstance(node, ast.ImportFrom):
                module = node.module or ""
                if node.level > 0:
                    # This is a relative import
                    module = "." * node.level + module
                imports.add(module)

        return imports

    def _file_to_module(self, file_path: str) -> Optional[str]:
        """Convert a file path to a module name."""
        try:
            # Get relative path from project root
            rel_path = os.path.relpath(file_path, str(self.root_dir))

            # Convert path to module
            return rel_path.replace(".py", "").replace(os.path.sep, ".")
        except Exception:
            return None

    def _find_module_file(self, module_name: str) -> Optional[str]:
        """
        Find the file for a module.
        """
        # Check if we already know this module
        if module_name in self.module_to_file:
            return self.module_to_file[module_name]

        # Try to find the file
        parts = module_name.split(".")
        for i in range(len(parts), 0, -1):
            prefix = ".".join(parts[:i])
            if prefix in self.module_to_file:
                return self.module_to_file[prefix]

        # Try to find the file on disk
        module_path = module_name.replace(".", os.path.sep)
        potential_paths = [
            os.path.join(str(self.root_dir), f"{module_path}.py"),
            os.path.join(str(self.root_dir), module_path, "__init__.py"),
        ]

        for path in potential_paths:
            if os.path.exists(path):
                return path

        return None

    def get_transitive_dependencies(self, module_names: List[str]) -> Set[str]:
        """
        Get all transitive dependencies of a list of modules.
        """
        visited = set()
        to_visit = set(module_names)

        while to_visit:
            current = to_visit.pop()
            if current in visited:
                continue

            visited.add(current)

            # Add dependencies to visit
            if current in self.module_dependencies:
                for dep in self.module_dependencies[current]:
                    if dep not in visited:
                        to_visit.add(dep)

        return visited

    def get_transitive_file_dependencies(self, file_paths: List[str]) -> Set[str]:
        """
        Get all transitive file dependencies of a list of files.
        """
        # Convert files to modules
        modules = []
        for file_path in file_paths:
            module = self.file_to_module.get(file_path)
            if module:
                modules.append(module)

        # Get transitive module dependencies
        dependent_modules = self.get_transitive_dependencies(modules)

        # Convert back to files
        dependent_files = set()
        for module in dependent_modules:
            file_path = self.module_to_file.get(module)
            if file_path:
                dependent_files.add(file_path)

        return dependent_files

    def get_impacted_modules(self, changed_files: List[str]) -> Set[str]:
        """
        Get all modules impacted by changes to the given files.
        """
        # First, get modules for changed files
        changed_modules = set()
        for file_path in changed_files:
            module = self.file_to_module.get(file_path)
            if module:
                changed_modules.add(module)

        # Then, find all modules that depend on these modules
        impacted_modules = set(changed_modules)
        for module, deps in self.module_dependencies.items():
            if any(dep in changed_modules for dep in deps):
                impacted_modules.add(module)

        # Get transitive dependencies
        return self.get_transitive_dependencies(list(impacted_modules))

    def get_impacted_files(self, changed_files: List[str]) -> Set[str]:
        """
        Get all files impacted by changes to the given files.
        """
        impacted_modules = self.get_impacted_modules(changed_files)

        # Convert modules to files
        impacted_files = set(changed_files)
        for module in impacted_modules:
            file_path = self.module_to_file.get(module)
            if file_path:
                impacted_files.add(file_path)

        return impacted_files
