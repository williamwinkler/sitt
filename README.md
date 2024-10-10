# SITT
SITT stands for **Si**mple **T**ime **T**racking. It's an application that allows users to track time on projects.<br>

SITT consists of:
  - **API** - which can be run on AWS Lambda
  - **CLI** - to interact with the API

## Deploy to AWS
Currently the API is deployed manually to AWS, but compiling and zipping the code as follows:
```bash
cargo lambda build --release --arm64 --output-format zip
```
Alternatively build it locally
```bash
cargo build --release
```

## Prerequistes
You will need:
- API key - every user has a unique API key
- SITT URL - The URL where the SITT API is deployed.

## CLI
```
$ sitt help

Use this CLI tool to interact with the (Si)mple (T)ime (T)racking API ⏱️

Usage: sitt <COMMAND>

Commands:
  start    Start time tracking on a project
  stop     Stop time tracking on a project
  project  Manage your projects
  time     Manage time on your projects
  config   Manage your configuration
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Setup
```bash
echo 'export PATH="$PATH:~/Documents/projects/sitt-api/target/release"' >> ~/.zshrc
```
