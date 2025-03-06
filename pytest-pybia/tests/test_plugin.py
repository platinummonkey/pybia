import pytest
from unittest.mock import patch, MagicMock

from pytest_pybia import plugin


def test_plugin_registered():
    """Test that the plugin is registered with pytest."""
    assert hasattr(plugin, "pytest_addoption")
    assert hasattr(plugin, "pytest_configure")
    assert hasattr(plugin, "pytest_collection_modifyitems")


def test_pytest_addoption():
    """Test that the plugin adds the expected options."""
    parser = MagicMock()
    plugin.pytest_addoption(parser)
    
    # Verify that the parser.addoption method was called with the expected arguments
    parser.addoption.assert_any_call(
        "--pybia", 
        action="store_true", 
        help="Enable PyBia impact analysis to skip tests not affected by changes"
    )


@patch("pytest_pybia.plugin.PyBiaClient")
def test_pytest_configure(mock_pybia_client):
    """Test that the plugin configures PyBia correctly."""
    config = MagicMock()
    config.getoption.return_value = True
    config.getini.return_value = "HEAD~1"
    
    plugin.pytest_configure(config)
    
    # Verify that PyBiaClient was initialized
    mock_pybia_client.assert_called_once()
    
    # Verify that the config object has a pybia attribute
    assert hasattr(config, "pybia")


@patch("pytest_pybia.plugin.PyBiaClient")
def test_pytest_collection_modifyitems(mock_pybia_client):
    """Test that the plugin modifies the test collection based on PyBia analysis."""
    config = MagicMock()
    config.getoption.return_value = True
    
    # Mock PyBia client instance
    mock_client_instance = MagicMock()
    mock_pybia_client.return_value = mock_client_instance
    
    # Mock the get_impacted_files method
    mock_client_instance.get_impacted_files.return_value = ["path/to/file1.py", "path/to/file2.py"]
    
    # Create mock test items
    item1 = MagicMock()
    item1.nodeid = "tests/test_file1.py::test_function1"
    item1.fspath = "tests/test_file1.py"
    
    item2 = MagicMock()
    item2.nodeid = "tests/test_file2.py::test_function2"
    item2.fspath = "tests/test_file2.py"
    
    items = [item1, item2]
    
    # Mock the is_test_impacted method to return True for item1 and False for item2
    mock_client_instance.is_test_impacted.side_effect = lambda item: item.nodeid == "tests/test_file1.py::test_function1"
    
    # Call the function under test
    plugin.pytest_collection_modifyitems(config, items)
    
    # Verify that item2 was marked as skipped
    item2.add_marker.assert_called_once()
    args, _ = item2.add_marker.call_args
    assert args[0].name == "skip"
    assert "not impacted by changes" in args[0].kwargs["reason"] 