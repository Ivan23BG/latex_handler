use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

// TODO: Replace this with your actual GitHub repository URL
const REPO_URL: &str = "https://github.com/Ivan23BG/latex_handler.git";



#[derive(Parser)]
#[command(name = "latex_handler")]
#[command(about = "Manages LaTeX templates and compiles documents", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Creates a new LaTeX project from templates
    New {
        /// The name of the new project directory
        name: String,
    },
    /// Updates the local template repository from GitHub
    Update,
    /// Compiles a LaTeX document
    Compile {
        /// Optional path to the .tex file or directory (defaults to current directory)
        path: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    // Resolve cross-platform data directory
    let proj_dirs = ProjectDirs::from("com", "YourName", "latex_handler")
        .expect("Could not determine home directory");
    
    // This is where we'll store the cloned GitHub repository
    let templates_dir = proj_dirs.data_dir();

    match &cli.command {
        Commands::New { name } => {
            let target_dir = env::current_dir().unwrap().join(name);

            // 1. Check if templates exist
            if !templates_dir.exists() || fs::read_dir(templates_dir).map(|mut i| i.next().is_none()).unwrap_or(true) {
                eprintln!("[Error] Templates not found in {}.", templates_dir.display());
                eprintln!(" [Info] Please run `latex_handler update` first to download them.");
                return;
            }

            println!(" [Info] Creating new project: '{}'...", name);
            
            // 2. Perform the recursive copy
            match copy_dir_all(templates_dir, &target_dir) {
                Ok(_) => println!(" [Info] Successfully created project '{}'!", name),
                Err(e) => eprintln!("[Error] Failed to create project: {}", e),
            }
        }
        
        Commands::Update => {
            // Check if the directory already has a .git folder inside it
            let git_dir = templates_dir.join(".git");

            if git_dir.exists() {
                println!(" [Info] Updating existing templates in {}...", templates_dir.display());
                
                // Run `git pull` from within the template directory
                let status = Command::new("git")
                    .arg("-C") // Tells git to run as if it was started in this directory
                    .arg(templates_dir)
                    .arg("pull")
                    .status()
                    .expect("Failed to execute git. Is git installed on your system?");

                if status.success() {
                    println!(" [Info] Templates updated successfully!");
                } else {
                    eprintln!("[Error] Failed to pull updates from GitHub.");
                }
            } else {
                println!(" [Info] Downloading templates for the first time to {}...", templates_dir.display());
                
                // Ensure the parent directory of our data dir exists before cloning
                if let Some(parent) = templates_dir.parent() {
                    fs::create_dir_all(parent).ok();
                }

                // Run `git clone <URL> <DIR>`
                let status = Command::new("git")
                    .arg("clone")
                    .arg(REPO_URL)
                    .arg(templates_dir)
                    .status()
                    .expect("Failed to execute git. Is git installed on your system?");

                if status.success() {
                    println!(" [Info] Templates downloaded successfully!");
                } else {
                    eprintln!("[Error] Failed to clone templates. Check your network or the repository URL.");
                }
            }
        }
        
        Commands::Compile { path } => {
            let target = path.as_deref().unwrap_or(".");
            println!(" [Info] Compiling target: {}", target);
            
            let status = Command::new("latexmk")
                .arg("-pdf")
                .arg(target)
                .status()
                .expect("Failed to execute compilation command. Is latexmk installed?");

            if status.success() {
                println!(" [Info] Compilation successful!");
            } else {
                eprintln!("[Error] Compilation failed.");
            }
        }
    }
}

/// Recursively copies a directory tree, ignoring `.git` folders.
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    // Create the destination directory if it doesn't exist
    fs::create_dir_all(dst)?;

    // Iterate through the contents of the source directory
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let file_name = entry.file_name();
        
        // CRITICAL: Do not copy the hidden git folder into new projects
        if file_name == ".git" {
            continue;
        }

        let dst_path = dst.join(file_name);

        if ty.is_dir() {
            // If it's a directory, recurse
            copy_dir_all(entry.path(), dst_path)?;
        } else {
            // If it's a file, just copy it
            fs::copy(entry.path(), dst_path)?;
        }
    }

    Ok(())
}