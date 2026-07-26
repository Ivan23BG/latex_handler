# My `latex_handler` tool

`latex_handler` is a cross-platform CLI tool designed to make compiling and managing LaTeX projects easier. It automates the compilation of LaTeX files, provides a structured way to handle multiple projects, and supports parallel compilation.

---

## Quick Install

Open your terminal and run the command for your OS. This will download the latest release binary, set up global execution, and pull the latest base templates.

### Linux / macOS
```bash
curl -sSL [https://raw.githubusercontent.com/Ivan23BG/latex_handler/main/install.sh](https://raw.githubusercontent.com/Ivan23BG/latex_handler/main/install.sh) | bash
```

### Windows (PowerShell)
```powershell
irm [https://raw.githubusercontent.com/Ivan23BG/latex_handler/main/install.ps1](https://raw.githubusercontent.com/Ivan23BG/latex_handler/main/install.ps1) | iex
```

Note: Make sure you restart your terminal after installation so that the `latex_handler` command is recognized.

### Non global installation
If you prefer not to install `latex_handler` globally, you can navigate to the release page, download the appropriate binary for your OS, and place it in a directory of your choice. Then, you can run it directly from that location.

---

## Usage

Once installed, you can use the `latex_handler` command in your terminal. Here are some common commands:


### Create a New LaTeX Project
```bash
latex_handler new [project_name]
```
- `project_name`: The name of your new LaTeX project. This will create a new directory with the specified name and populate it with a basic LaTeX template.


### Compile LaTeX Files
```bash
latex_handler compile [path]
```
- `path`: Optional. The path to the directory containing your LaTeX files. If not provided, the current directory will be used.


### Update Base Templates
```bash
latex_handler update
```
This command will pull the latest base templates from the repository and update your local templates. It ensures that you have the most recent improvements and fixes for your LaTeX projects.

---

## Expected Directory Structure

When you create a new LaTeX project using `latex_handler`, the expected directory structure is as follows:

```
project_name/
├── src/                  # Place your source code & TeX files here
│   ├── assets/           # Global libraries (math, environments, tikz, etc.)
│   │                     # These are provided by the base template
│   ├── beamer/           # Slides presentation source files
│   ├── rapport/          # Main report/paper source files
│   └── other_files/      # Other document with source files
├── build/                # Intermediate build artifacts (ignored by git)
├── pdfs/                 # Output PDF files copied directly here
└── logs/                 # Log files are stored here for debugging purposes
```


---
## Prerequisites

To compile projects, you need to have the following installed on your system:
- **LaTeX Distribution**: Ensure you have a LaTeX distribution installed (e.g., TeX Live, MiKTeX, or MacTeX).
- **latexmk**: A Perl script that automates the process of generating a LaTeX document. It is usually included with most LaTeX distributions. Make sure it is available in your system's PATH.
- **Git**: Required for updating base templates and managing project versions.

---
## Building from Source
If you want to build `latex_handler` from source, follow these steps:

1. Clone the repository:
   ```bash
   git clone https://github.com/Ivan23BG/latex_handler.git
   ```

2. Navigate to the project directory:
   ```bash
   cd latex_handler
   ```

3. Build the project using Cargo:
   ```bash
   cargo build --release
   ```
4. The compiled binary will be located in the `target/release` directory. You can move it to a directory in your PATH for easier access.