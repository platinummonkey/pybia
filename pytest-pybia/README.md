# pytest-pybia

A pytest plugin that integrates with PyBia to skip tests not impacted by code changes.

## Features

- **Smart Test Selection**: Only run tests affected by your code changes
- **Transitive Dependency Analysis**: Understands how changes propagate through your codebase
- **CI Optimization**: Dramatically reduce CI time by skipping unnecessary tests
- **Impact Summary**: Get a clear summary of which services and modules are impacted
- **Configurable**: Easily customize behavior through pytest options

## Installation

```bash
pip install pytest-pybia
```

## Requirements

- Python 3.7+
- pytest 6.0+
- PyBia installed and available in PATH

## Usage

Basic usage:

```bash
# Run only tests affected by recent changes
pytest --pybia

# Run only tests affected by changes since a specific git commit
pytest --pybia --pybia-base-commit=HEAD~5

# Run only tests affected by changes in specific files
pytest --pybia --pybia-files=src/module1.py,src/module2.py

# Force running all tests regardless of impact analysis
pytest --pybia --pybia-force-all
```

## Configuration

You can configure pytest-pybia in your `pytest.ini`, `pyproject.toml`, or `conftest.py`:

```ini
# pytest.ini
[pytest]
pybia_enabled = true
pybia_base_commit = origin/main
pybia_config_file = pybia.toml
pybia_summary = true
```

```toml
# pyproject.toml
[tool.pytest.ini_options]
pybia_enabled = true
pybia_base_commit = "origin/main"
pybia_config_file = "pybia.toml"
pybia_summary = true
```

```python
# conftest.py
def pytest_addoption(parser):
    parser.addini('pybia_enabled', 'Enable PyBia impact analysis', default='true')
    parser.addini('pybia_base_commit', 'Git base commit for comparison', default='HEAD~1')
```

## How It Works

1. The plugin runs PyBia to determine which files and services are impacted by changes
2. It analyzes the dependency graph to find all tests affected by those changes
3. During test collection, it skips tests that aren't impacted
4. After the test run, it provides a summary of impacted services and modules

## Example Output

```
============================= PyBia Impact Summary =============================
Detected changes in 3 files:
  - src/auth/models.py
  - src/auth/utils.py
  - tests/auth/test_models.py

Impacted services:
  - auth-service (3 files)
  - api-service (1 file, via dependency)

Running 12/45 tests (33 skipped due to no impact)
===============================================================================
```

## License

This project is licensed under the MIT License - see the LICENSE file for details. 