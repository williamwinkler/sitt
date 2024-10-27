# SITT
SITT is a **Si**mple **T**ime **T**racking application designed for tracking time on projects ⏱️

## Overview
SITT consists of:
  - **API**<br>
  A RESTful HTTP API to manage projects, time logs, and users. Designed for deployment on AWS Lambda using DynamoDB (free tier)

  - **CLI**<br>
    A command-line tool for interacting with the API, making time tracking straightforward.

Both written 100% in Rust 🦀.

## Usage
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
  user     [ADMIN ONLY] Manage users
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
## Example commands:
```bash
# Create a project
sitt project create --name my-project

# Start tracking time
sitt start --name my-project

# Stop tracking time
sitt stop -n my-project
```
### Demo:

[![asciicast](https://asciinema.org/a/BrUqWZ2s8tjN3qV9YNjNWuZW8.svg)](https://asciinema.org/a/BrUqWZ2s8tjN3qV9YNjNWuZW8)

## Get Started
To set up SITT, follow these steps:
1. **Deploy the API.**
2. **Install the CLI**
3. **Authenticate**

### 1. Deploy API
> *If you will be using an already deployed API, skip this step.*

The API can be deployed on AWS Lambda or run as a traditional server process. On startup, it will automatically create necessary DynamoDB tables and an initial admin user (named `admin` with API key `admin`).
> **Important: Use the default `admin` user to create another ADMIN user and then delete the default `admin` account.**

#### Deploying to AWS Lambda
1. Compile and zip the API for deployment on Lambda:

    ```bash
    cargo lambda build --release --arm64 --output-format zip
    ```
    This creates a file `target/lambda/sitt-api/bootstrap.zip`.<br>

2. In AWS:
    - Create a Lambda Function and upload the zip file.

    - Create a public URL, such as `https://<random>.lambda-url.<region>.on.aws`, for the Lambda Function. This URL will be used by the CLI.
    This URL will be used by the CLI.

#### Traditional deployment
1. Build the API locally:
    ```bash
    cargo build --release
    ```
2. Ensure to set the environment variables from [.env.example](.env.example) to configure the API.



### 2. Install CLI
Choose your platform and follow the instructions below:
<details>
<summary>MacOS</summary>
<br>

1. Download the sitt and allow it to be executed:

    ```bash
    sudo curl -L "https://github.com/williamwinkler/sitt/releases/latest/download/sitt-macos" -o /usr/local/bin/sitt
    sudo chmod +x /usr/local/bin/sitt
    ```

2. Verify installation:
    ```bash
    sitt --help
    ```

3. Troubleshoot

    If macOS quarantines the binary, allow execution by running:
    ```bash
    sudo xattr -rd com.apple.quarantine /usr/local/bin/sitt
    ```
</details>

<details>
<summary>Linux</summary>
<br>

1. Download the sitt and allow it to be executed:

    ```bash
    mkdir -p ~/.local/bin || true
    curl -L "https://github.com/williamwinkler/sitt/releases/latest/download/sitt-linux" -o ~/.local/bin/sitt
    chmod +x ~/.local/bin/sitt
    export PATH="$HOME/.local/bin:$PATH"  # Ensure ~/.local/bin is in your PATH
    ```

2. Verify installation
    ```bash
    sitt --help
    ```
</details>


<details>
<summary>Windows</summary>
<br>

1. Download `sitt`

    Using PowerShell, download the `sitt` executable to your local `bin` folder:
    ```powershell
    if (-not (Test-Path "$Env:USERPROFILE\bin")) { New-Item -ItemType Directory -Path "$Env:USERPROFILE\bin" | Out-Null }
    Invoke-WebRequest -Uri "https://github.com/williamwinkler/sitt/releases/latest/download/sitt-windows.exe" -OutFile "$Env:USERPROFILE\bin\sitt.exe"
    ```

2. Ensure `$Env:USERPROFILE\bin` is in your PATH

    To make sure `sitt` is accessible from any PowerShell session, add `$Env:USERPROFILE\bin` to your PATH environment variable.

- **Permanently** (for all future sessions):
  ```powershell
  [Environment]::SetEnvironmentVariable("Path", "$env:Path;$Env:USERPROFILE\bin", [EnvironmentVariableTarget]::User)
  ```

    > **Note**: Adding it permanently ensures `sitt` is accessible from any terminal in the future.

3. Restart PowerShell (if needed)

    If the `sitt` command is not immediately recognized, close and reopen PowerShell or any terminal you are using to refresh the environment variables.
   
4. Verify Installation

    After adding the path, verify that `sitt` is properly installed by running:
    ```powershell
    sitt --help
    ```

    If you see the help information, the installation is successful!
</details>


### 3. Authenticate
You will need the API URL and an API key to authenticate. Admin users can create other users using:
```bash
sitt user create
```

To authenticate:
```bash
sitt start
```
Then, enter the API URL and your API key when prompted.

Once authenticated, you’re ready to start tracking! ✅
