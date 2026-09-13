use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use rayon::prelude::*;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use walkdir::WalkDir;

const REPO_URL: &str = "https://github.com/Ivan23BG/LaTeX_Template.git";



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
    let proj_dirs = ProjectDirs::from("com", "Ivan23", "latex_handler")
        .expect("Could not determine home directory");
    
    // This is where we'll store the cloned GitHub repository
    let templates_dir = proj_dirs.data_dir();

    match &cli.command {
        Commands::New { name } => {
            let target_dir = env::current_dir().unwrap().join(name);

            // 1. Check if templates exist
            if !templates_dir.exists() || fs::read_dir(templates_dir).map(|mut i| i.next().is_none()).unwrap_or(true) {
                eprintln!("[Error] Templates not found in {}.", templates_dir.display());
                eprintln!(" [Warn] Please run `latex_handler update` first to download them.");
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
            let raw_path = match path {
                Some(p) => PathBuf::from(p),
                Option::None => env::current_dir().expect("Failed to get current directory"),
            };

            // Ensure project_root is an absolute path so latexmk receives absolute -outdir flags
            let project_root = make_absolute(&raw_path);

            let src_dir = project_root.join("src");
            let build_root = project_root.join("build");
            let pdf_root = project_root.join("pdfs");
            let log_root = project_root.join("logs");

            let exclude_patterns = vec!["legacy", "templates", "tmp", "temp"];

            // 1. Discover all *_main.tex files
            let tex_files = find_main_tex_files(&src_dir, "_main.tex", &exclude_patterns);

            if tex_files.is_empty() {
                println!(" [Warn] No matching '_main.tex' files found in '{}'", src_dir.display());
                return;
            }

            println!(" [Info] Found the following files to compile:");
            for f in &tex_files {
                println!("   {}", f.display());
            }

            // 2. Determine thread count (up to 8, or physical CPU count)
            let num_cpus = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1);
            let max_workers = std::cmp::min(8, num_cpus);

            println!("\n [Info] Compiling with {} parallel workers...\n", max_workers);

            // Configure thread pool
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(max_workers)
                .build()
                .expect("Failed to build thread pool");

            // 3. Compile in parallel
            let (successes, failures): (Vec<PathBuf>, Vec<PathBuf>) = pool.install(|| {
                tex_files
                    .into_par_iter()
                    .partition(|tex| {
                        compile_latex(tex, &src_dir, &build_root, &pdf_root, &log_root)
                    })
            });

            // 4. Output Summary
            println!("\n===== Compilation Summary =====");
            if !successes.is_empty() {
                println!(" [Info] Successfully compiled: {}", successes.len());
                for f in &successes {
                    println!("    {}", f.display());
                }
            }

            if !failures.is_empty() {
                println!("\n[Error] Failed to compile: {}", failures.len());
                for f in &failures {
                    println!("    {}", f.display());
                }
                std::process::exit(1);
            } else {
                println!("\n [Info] All files compiled successfully!");
            }
        }
    }
}

/// Finds all files ending with a given suffix in `root`, excluding matched path patterns
fn find_main_tex_files(root: &Path, suffix: &str, exclude_patterns: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !root.exists() {
        return files;
    }

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.ends_with(suffix) {
                    let path_str = path.to_string_lossy();
                    if !exclude_patterns.iter().any(|pat| path_str.contains(pat)) {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }
    }

    files
}

/// Mirrors src/... directory hierarchy under target root dir
fn mirror_under(root_dir: &Path, src_dir: &Path, tex_file: &Path) -> io::Result<PathBuf> {
    let parent = tex_file.parent().unwrap_or(src_dir);
    let rel = parent.strip_prefix(src_dir).unwrap_or(Path::new(""));
    let target = root_dir.join(rel);
    fs::create_dir_all(&target)?;
    Ok(target)
}

/// Compiles a single .tex file, moves PDF to /pdfs and log to /logs
fn compile_latex(
    tex_file: &Path,
    src_dir: &Path,
    build_root: &Path,
    pdf_root: &Path,
    log_root: &Path,
) -> bool {
    let job_name = match tex_file.file_stem().and_then(|s| s.to_str()) {
        Some(stem) => stem,
        Option::None => return false,
    };

    let build_dir = match mirror_under(build_root, src_dir, tex_file) {
        Ok(d) => d,
        Err(_) => return false,
    };
    let pdf_dir = match mirror_under(pdf_root, src_dir, tex_file) {
        Ok(d) => d,
        Err(_) => return false,
    };
    let log_dir = match mirror_under(log_root, src_dir, tex_file) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let tex_filename = format!("{}.tex", job_name);
    let outdir_arg = format!("-outdir={}", build_dir.display());
	let log_stdout = fs::File::create(log_dir.join(format!("{}.stdout.log", job_name))).ok();
	let log_stderr = fs::File::create(log_dir.join(format!("{}.stderr.log", job_name))).ok();

	let status = Command::new("latexmk")
	    .arg("-pdf")
	    .arg("-synctex=1")
	    .arg("-shell-escape")
	    .arg("-interaction=nonstopmode")
	    .arg("-halt-on-error")
	    .arg(&outdir_arg)
	    .arg(&tex_filename)
	    .current_dir(tex_file.parent().unwrap_or_else(|| Path::new(".")))
	    .stdout(log_stdout.map_or(Stdio::null(), Stdio::from))
	    .stderr(log_stderr.map_or(Stdio::null(), Stdio::from))
	    .status();

    let success = match status {
        Ok(s) => s.success(),
        Err(_) => false,
    };

    // Copy PDF
    let pdf_src = build_dir.join(format!("{}.pdf", job_name));
    if pdf_src.exists() {
        let _ = fs::copy(&pdf_src, pdf_dir.join(format!("{}.pdf", job_name)));
    }
    
    // Copy SyncTeX file
    let synctex_src = build_dir.join(format!("{}.synctex.gz", job_name));
    if synctex_src.exists() {
        let _ = fs::copy(&synctex_src, pdf_dir.join(format!("{}.synctex.gz", job_name)));
    }

    // Move log file
    let log_src = build_dir.join(format!("{}.log", job_name));
    if log_src.exists() {
        let dest_log = log_dir.join(format!("{}.log", job_name));
        let _ = fs::rename(&log_src, &dest_log).or_else(|_| {
            fs::copy(&log_src, &dest_log).and_then(|_| fs::remove_file(&log_src))
        });
    }

    success
}

/// Recursively copies a directory tree, ignoring `.git` folders.
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let file_name = entry.file_name();
        
        if file_name == ".git" {
            continue;
        }

        let dst_path = dst.join(file_name);

        if ty.is_dir() {
            copy_dir_all(entry.path(), dst_path)?;
        } else {
            fs::copy(entry.path(), dst_path)?;
        }
    }

    Ok(())
}

/// Ensures a path is absolute without using canonicalize (which adds unwanted UNC prefixes on Windows)
fn make_absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        match env::current_dir() {
            Ok(cwd) => cwd.join(path),
            Err(_) => path.to_path_buf(),
        }
    }
}
