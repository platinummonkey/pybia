# PyBia - Python Build Impact Analysis

PyBia is a build impact analysis tool designed specifically for Python codebases. It intelligently analyzes your code to determine which parts are affected by changes, helping you optimize CI/CD pipelines and understand the impact of code modifications across your project.

## Features

- **Smart Impact Detection**: Automatically identifies which Python modules, packages, and services are affected by code changes
- **Dependency Graph Analysis**: Builds a comprehensive dependency graph of your Python codebase
- **CI Pipeline Optimization**: Only run tests and builds for components affected by changes
- **Monorepo Support**: Perfect for monorepos with multiple Python services or packages
- **Real-time Monitoring**: Watch for file changes and instantly determine their impact
- **Customizable Rules**: Configure detection rules to match your project structure

## Installation

```bash
cargo install pybia
```

Or build from source:

```bash
git clone https://github.com/platinummonkey/pybia.git
cd pybia
cargo build --release
```

## Usage

```bash
# Analyze impact of changes and run tests only for affected components
pybia --paths /path/to/project -- pytest {affected_services}

# Watch multiple directories in a monorepo
pybia --paths /path/to/monorepo/services --service-format name -- ./run-affected-tests.sh

# Integrate with CI using a configuration file
pybia --paths /path/to/project --services-config ci-services.toml -- make test-affected

# Output affected components for use in other scripts
pybia --paths /path/to/project --service-format name-path > affected_components.txt
```

## CI Integration

PyBia excels at optimizing CI/CD pipelines by:

1. Analyzing which components are affected by changes in a pull request
2. Running tests only for affected components
3. Building and deploying only what changed
4. Providing clear impact reports for code reviews

Example GitHub Actions integration:

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Determine affected services
        run: pybia --paths . --service-format name > affected.txt
      - name: Run tests for affected services only
        run: cat affected.txt | xargs -I{} pytest {}
```

## Service Configuration

Define your project structure in a TOML configuration file:

```toml
[[services]]
name = "auth-service"
path = "/path/to/auth"
include_paths = ["src", "tests"]
exclude_paths = ["docs"]

[[services]]
name = "api-service"
path = "/path/to/api"
```

## How It Works

PyBia works by:

1. Building a dependency graph of your Python codebase
2. Analyzing imports, requirements, and package dependencies
3. Determining the impact radius of code changes
4. Identifying which components need to be tested or rebuilt
5. Providing actionable output for CI systems or developers

## Use Cases

- **CI Optimization**: Reduce CI time by up to 90% by only testing what's affected
- **Change Impact Analysis**: Understand how code changes propagate through your system
- **Dependency Visualization**: Gain insights into your codebase's structure
- **Monorepo Management**: Make large Python monorepos manageable and efficient

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.


## Note

This is purely experimental and for fun at this moment. This was entirely developed using dialogue-drive-development with cursor using claude-3.5 & claude-3.7-sonnet.  This note section is the only piece I've actually written by hand.
