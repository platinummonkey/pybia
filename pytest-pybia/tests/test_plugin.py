from unittest.mock import MagicMock, patch

from pytest_pybia import plugin
from pytest_pybia.plugin import _PLUGIN_KEY


def test_plugin_registered():
    """Test that the plugin is registered with pytest."""
    assert hasattr(plugin, "pytest_addoption")
    assert hasattr(plugin, "pytest_configure")
    assert hasattr(plugin, "pytest_collection_modifyitems")


def test_pytest_addoption():
    """Test that the plugin adds the expected options."""
    parser = MagicMock()
    group = MagicMock()
    parser.getgroup.return_value = group
    plugin.pytest_addoption(parser)

    # Verify that the group.addoption method was called with the --pybia flag
    group.addoption.assert_any_call(
        "--pybia",
        action="store_true",
        dest="pybia_enabled",
        default=False,
        help="Enable PyBia impact analysis to skip tests not affected by changes",
    )


@patch("pytest_pybia.plugin.PyBiaPlugin")
def test_pytest_configure(mock_pybia_plugin_class):
    """Test that the plugin configures PyBia correctly when enabled."""
    mock_plugin_instance = MagicMock()
    mock_plugin_instance.enabled = True
    mock_pybia_plugin_class.return_value = mock_plugin_instance

    config = MagicMock()
    config.getoption.return_value = True
    config.getini.side_effect = lambda key: {
        "pybia_enabled": "true",
        "pybia_base_commit": "HEAD~1",
        "pybia_config_file": "",
        "pybia_summary": "true",
    }.get(key, "")

    plugin.pytest_configure(config)

    # Verify that PyBiaPlugin was instantiated
    mock_pybia_plugin_class.assert_called_once()

    # Verify the instance was registered under the internal key
    config.pluginmanager.register.assert_called_once_with(
        mock_plugin_instance, _PLUGIN_KEY
    )


@patch("pytest_pybia.plugin.PyBiaPlugin")
def test_pytest_collection_modifyitems(mock_pybia_plugin_class):
    """Test that the plugin modifies the test collection based on PyBia analysis."""
    mock_plugin_instance = MagicMock()
    mock_plugin_instance.enabled = True
    mock_plugin_instance.force_all = False
    mock_pybia_plugin_class.return_value = mock_plugin_instance

    config = MagicMock()
    config.pluginmanager.get_plugin.return_value = mock_plugin_instance

    # Create mock test items
    item1 = MagicMock()
    item1.nodeid = "tests/test_file1.py::test_function1"
    item1.fspath = "tests/test_file1.py"

    item2 = MagicMock()
    item2.nodeid = "tests/test_file2.py::test_function2"
    item2.fspath = "tests/test_file2.py"

    items = [item1, item2]

    # Mock is_test_impacted: True for item1, False for item2
    mock_plugin_instance.is_test_impacted.side_effect = lambda item: (
        item.nodeid == "tests/test_file1.py::test_function1"
    )

    # Call the function under test
    plugin.pytest_collection_modifyitems(config, items)

    # Verify get_plugin used the internal registration key
    config.pluginmanager.get_plugin.assert_called_with(_PLUGIN_KEY)

    # Verify that item1 (impacted) was NOT marked as skipped
    item1.add_marker.assert_not_called()

    # Verify that item2 (not impacted) was marked as skipped
    item2.add_marker.assert_called_once()
    args, _ = item2.add_marker.call_args
    assert args[0].name == "skip"
    assert "not impacted" in args[0].kwargs["reason"].lower()
