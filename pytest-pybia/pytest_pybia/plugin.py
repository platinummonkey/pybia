import os
import subprocess
import json
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple, Any

import pytest
from _pytest.config import Config
from _pytest.config.argparsing import Parser
from _pytest.main import Session
from _pytest.python import Module, Function
from _pytest.reports import TestReport
from _pytest.terminal import TerminalReporter

from .impact import ImpactAnalyzer


class PyBiaPlugin:
    """
    Pytest plugin that integrates with PyBia to skip tests not impacted by code changes.
    """

    def __init__(self):
        self.enabled = False
        self.base_commit = "HEAD~1"
        self.config_file = None
        self.show_summary = True
        self.force_all = False
        self.specified_files = None
        
        # Impact analyzer
        self.impact_analyzer = None
        
        # Statistics
        self.total_tests = 0
        self.run_tests = 0
        self.skipped_tests = 0

    def initialize(self, rootdir: Path) -> None:
        """Initialize the plugin with the given root directory."""
        self.impact_analyzer = ImpactAnalyzer(rootdir)
        self.impact_analyzer.scan_codebase()
        
        # Parse specified files if provided
        changed_files = None
        if self.specified_files:
            changed_files = [
                os.path.join(str(rootdir), f.strip())
                for f in self.specified_files.split(",")
                if f.strip()
            ]
        
        # Analyze impact
        self.impact_analyzer.analyze_impact(
            config_file=self.config_file,
            changed_files=changed_files,
            base_commit=self.base_commit if not changed_files else None
        )

    def _is_test_impacted(self, item: Function) -> bool:
        """Determine if a test is impacted by changes."""
        if self.force_all:
            return True
            
        # Get the module name and file path
        module_name = item.module.__name__
        file_path = str(item.path)
        
        # Check if the test file itself is impacted
        if self.impact_analyzer.is_file_impacted(file_path):
            return True
            
        # Check if the test module is impacted
        if self.impact_analyzer.is_module_impacted(module_name):
            return True
            
        return False

    def print_summary(self, tr: TerminalReporter) -> None:
        """Print a summary of the impact analysis."""
        if not self.show_summary:
            return
            
        tr.write_sep("=", "PyBia Impact Summary")
        
        # Show changed files
        changed_files = self.impact_analyzer.changed_files
        tr.write_line(f"Detected changes in {len(changed_files)} files:")
        for file in changed_files[:5]:  # Show only first 5 files
            tr.write_line(f"  - {os.path.relpath(file, str(tr.config.rootdir))}")
        if len(changed_files) > 5:
            tr.write_line(f"  - ... and {len(changed_files) - 5} more")
        
        # Show impacted services
        tr.write_line("")
        tr.write_line(f"Impacted services:")
        for service_name, service_path in self.impact_analyzer.impacted_services:
            file_count = sum(1 for f in self.impact_analyzer.impacted_files if f.startswith(service_path))
            tr.write_line(f"  - {service_name} ({file_count} files)")
        
        # Show test statistics
        tr.write_line("")
        tr.write_line(f"Running {self.run_tests}/{self.total_tests} tests "
                     f"({self.skipped_tests} skipped due to no impact)")
        
        tr.write_sep("=", "")


def pytest_addoption(parser: Parser) -> None:
    """Add PyBia-specific command line options to pytest."""
    group = parser.getgroup("pybia", "PyBia impact analysis")
    group.addoption(
        "--pybia",
        action="store_true",
        dest="pybia_enabled",
        default=False,
        help="Enable PyBia impact analysis to skip tests not affected by changes",
    )
    group.addoption(
        "--pybia-base-commit",
        action="store",
        dest="pybia_base_commit",
        default=None,
        help="Git base commit to compare changes against (default: HEAD~1)",
    )
    group.addoption(
        "--pybia-config-file",
        action="store",
        dest="pybia_config_file",
        default=None,
        help="Path to PyBia configuration file",
    )
    group.addoption(
        "--pybia-no-summary",
        action="store_false",
        dest="pybia_summary",
        default=True,
        help="Disable PyBia impact summary",
    )
    group.addoption(
        "--pybia-force-all",
        action="store_true",
        dest="pybia_force_all",
        default=False,
        help="Force running all tests regardless of impact analysis",
    )
    group.addoption(
        "--pybia-files",
        action="store",
        dest="pybia_files",
        default=None,
        help="Comma-separated list of files to analyze for impact",
    )
    
    # Add ini options
    parser.addini("pybia_enabled", "Enable PyBia impact analysis", default="false")
    parser.addini("pybia_base_commit", "Git base commit for comparison", default="HEAD~1")
    parser.addini("pybia_config_file", "Path to PyBia configuration file")
    parser.addini("pybia_summary", "Show PyBia impact summary", default="true")


@pytest.hookimpl
def pytest_configure(config: Config) -> None:
    """Configure the PyBia plugin."""
    plugin = PyBiaPlugin()
    
    # Check if plugin is enabled
    plugin.enabled = (
        config.getoption("pybia_enabled") or
        config.getini("pybia_enabled").lower() == "true"
    )
    
    if not plugin.enabled:
        return
    
    # Register plugin
    config.pluginmanager.register(plugin, "pybia")
    
    # Get configuration
    plugin.base_commit = (
        config.getoption("pybia_base_commit") or
        config.getini("pybia_base_commit") or
        "HEAD~1"
    )
    plugin.config_file = (
        config.getoption("pybia_config_file") or
        config.getini("pybia_config_file")
    )
    plugin.show_summary = (
        config.getoption("pybia_summary") and
        config.getini("pybia_summary").lower() != "false"
    )
    plugin.force_all = config.getoption("pybia_force_all")
    plugin.specified_files = config.getoption("pybia_files")
    
    # Initialize the plugin
    plugin.initialize(Path(config.rootdir))


@pytest.hookimpl(trylast=True)
def pytest_collection_modifyitems(config: Config, items: List[Function]) -> None:
    """Skip tests that aren't impacted by changes."""
    plugin = config.pluginmanager.get_plugin("pybia")
    if not plugin or not plugin.enabled:
        return
    
    plugin.total_tests = len(items)
    
    if plugin.force_all:
        plugin.run_tests = plugin.total_tests
        plugin.skipped_tests = 0
        return
    
    skip_marker = pytest.mark.skip(reason="Not impacted by changes according to PyBia")
    for item in items:
        if not plugin._is_test_impacted(item):
            item.add_marker(skip_marker)
            plugin.skipped_tests += 1
        else:
            plugin.run_tests += 1


@pytest.hookimpl(trylast=True)
def pytest_terminal_summary(terminalreporter: TerminalReporter, exitstatus: int, config: Config) -> None:
    """Add PyBia impact summary to the terminal report."""
    plugin = config.pluginmanager.get_plugin("pybia")
    if plugin and plugin.enabled:
        plugin.print_summary(terminalreporter) 