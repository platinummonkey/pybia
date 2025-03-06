# PyBia - Python Service Dependency Watcher

PyBia is a file watcher tool designed specifically for Python microservice architectures. It monitors your codebase for changes and intelligently identifies which services are affected by those changes, helping you optimize your development and testing workflows.

## Features

- **Service Detection**: Automatically detects Python services in your codebase by identifying setup.py and pyproject.toml files
- **Dependency Tracking**: Analyzes import statements and dependency files to build a dependency graph between services
- **File Watching**: Monitors file changes in real-time and identifies affected services
- **Customizable**: Configure service definitions manually or let PyBia detect them automatically
- **Command Execution**: Run custom commands when files change

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
# Watch a directory and run tests when files change
pybia --paths /path/to/project -- pytest

# Watch multiple directories
pybia --paths /path/to/service1 /path/to/service2 -- cargo test

# Specify a services configuration file
pybia --paths /path/to/project --services-config services.toml -- make test

# Output affected services in different formats
pybia --paths /path/to/project --service-format name-path
```

## Service Configuration

You can define services explicitly in a TOML configuration file:

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

1. Detecting Python services in your codebase
2. Analyzing import statements and dependency files to build a dependency graph
3. Watching for file changes
4. Determining which services are affected by changes
5. Running specified commands or outputting affected services

## Use Cases

- **Monorepo Development**: Only test services affected by your changes
- **CI/CD Optimization**: Only build and deploy services that have changed
- **Dependency Analysis**: Understand the relationships between your services

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.


## Note

This is purely experimental and for fun at this moment. This was entirely developed using dialogue-drive-development with cursor using claude-3.5 & claude-3.7-sonnet.  This note section is the only piece I've actually written by hand.
