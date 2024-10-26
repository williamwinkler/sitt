# SITT
SITT is an application that allows users to track time on projects ⏱️ <br>
SITT stands for **Si**mple **T**ime **T**racking. <br>

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
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```bash
# Create a project
sitt project create --name my-project

# Start tracking time
sitt start --name my-project

# Stop tracking time
sitt stop -n my-project
```

[![asciicast](https://asciinema.org/a/BrUqWZ2s8tjN3qV9YNjNWuZW8.svg)](https://asciinema.org/a/BrUqWZ2s8tjN3qV9YNjNWuZW8)

### Overview
SITT consists of:
  - **API**<br>
  A restful HTTP API that exposes a suite of endpoints to manage projects, time logging and users.<br>
  Meant to be run on AWS Lambda using the free tier of the serverless AWS database service DynamoDB.

  - **CLI**<br>
  Command line interface to interact with API.

Both written 100% in Rust.

## Get started
Follow these steps to get started:
1. (optional) Deploy the API.
2. Install the CLI.
3. Authenticate.

### 1. Deploy API

The API is designed to be deployed on AWS Lambda, but it can also be deployed as a traditional OS process.
The API will **automatically** create the necessary tables in DynamoDB upon upstart and create an admin user with with username `admin` & API key: `admin`.
> Important: Use the default `admin` user to create a another ADMIN user and use that to delete the default one.

#### AWS Lambda
To deploy it to AWS Lambda, you need to compile and zip the code:
```bash
cargo lambda build --release --arm64 --output-format zip
```
This creates a file `target/lambda/sitt-api/bootstrap.zip`.<br>

In the AWS portal, create a Lambda Function and upload the zip file.

Create a public reachable URL for the Lambda function like `https://<random>.lambda-url.<region>.on.aws`.<br>
This URL will be used by the CLI.

#### Traditional deployment
Alternatively build it locally
```bash
cargo build --release
```

Ensure include the environment variables found in [.env.example](.env.example).

### 2. Install CLI
Download and install the CLI for you platform:
<details>
<summary>MacOS</summary>
<br>
Download the sitt and allow it to be executed:

```bash
curl -L "https://github.com/williamwinkler/sitt/releases/latest/download/sitt-macos" -o ~/.local/bin/sitt
chmod +x ~/.local/bin/sitt
```

Verify installation
```bash
sitt --help
```

It's possible that MacOS will quarantine the binary. To allow it to execute run:
```bash
sudo xattr -rd com.apple.quarantine /usr/local/bin/sitt
```
</details>

<details>
<summary>Linux</summary>
<br>
Download the sitt and allow it to be executed:

```bash
curl -L "https://github.com/williamwinkler/sitt/releases/latest/download/sitt-linux" -o ~/.local/bin/sitt
chmod +x ~/.local/bin/sitt
export PATH="$HOME/.local/bin:$PATH"  # Ensure ~/.local/bin is in your PATH
```
Verify installation
```bash
sitt --help
```
</details>

<details>
<summary>Windows</summary>
<br>

*Using PowerShell*

Step 1: Download the Binary
```powershell
Invoke-WebRequest -Uri "https://github.com/williamwinkler/sitt/releases/latest/download/sitt-windows.exe" -OutFile "$Env:USERPROFILE\bin\sitt.exe"
```
Ensure `$Env:USERPROFILE\bin` is in your PATH

Verify installation
```powershell
sitt --help
```
</details>

### 3. Authenticate
You will need the URL for the API and an API key for your user.
An ADMIN can create users with `sitt user create`.

To authenticate run:
```bash
sitt start
```

Fill in the URL and API key

Now you should be good to go ✅
