use clap::{Parser, Subcommand, ValueEnum};
use directories::ProjectDirs;
use rayon::prelude::*;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command};
use walkdir::WalkDir;

const TEMPLATE_REPO_URL: &str = "https://github.com/Ivan23BG/latex_handler_template.git";
const INSTALL_SCRIPT_URL: &str = "https://raw.githubusercontent.com/Ivan23BG/latex_handler/main/install.sh";

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Theme {
    Light,
    Dark,
    All,
}

impl Theme {
    fn active_themes(&self) -> Vec<&'static str> {
        match self {
            Theme::Light => vec!["light"],
            Theme::Dark => vec!["dark"],
            Theme::All => vec!["light", "dark"],
        }
    }
}

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
    /// Updates both the LaTeX templates and the latex_handler CLI binary
    Update,
    /// Compiles LaTeX documents in parallel for specified theme(s)
    Compile {
        /// Optional path to the .tex file or project directory (defaults to current directory)
        path: Option<String>,

        /// Theme to compile: light, dark, or all (defaults to light)
        #[arg(short, long, value_enum, default_value_t = Theme::Light)]
        theme: Theme,
    },
}

#[derive(Clone, Debug)]
struct CompileJob {
    tex_file: PathBuf,
    theme: &'static str,
}

fn main() {
    let cli = Cli::parse();

    let proj_dirs = ProjectDirs::from("com", "Ivan23", "latex_handler")
        .expect("Could not determine user data directory");
    let templates_dir = proj_dirs.data_dir();

    match &cli.command {
        Commands::New { name } => {
            let target_dir = env::current_dir().unwrap().join(name);

            if !templates_dir.exists() || fs::read_dir(templates_dir).map(|mut i| i.next().is_none()).unwrap_or(true) {
                eprintln!("[Error] Templates not found in {}.", templates_dir.display());
                eprintln!(" [Warn] Please run `latex_handler update` first to download them.");
                return;
            }

            println!(" [Info] Creating new project: '{}'...", name);
            
            match copy_dir_all(templates_dir, &target_dir) {
                Ok(_) => println!(" [Info] Successfully created project '{}'!", name),
                Err(e) => eprintln!("[Error] Failed to create project: {}", e),
            }
        }
        
        Commands::Update => {
            println!("========================================");
            println!(" [1/2] Updating Template Repository...");
            println!("========================================");
            update_templates(templates_dir);

            println!("\n========================================");
            println!(" [2/2] Updating latex_handler CLI...");
            println!("========================================");
            update_self();
        }
        
        Commands::Compile { path, theme } => {
            let raw_path = match path {
                Some(p) => PathBuf::from(p),
                None => env::current_dir().expect("Failed to get current directory"),
            };

            let project_root = make_absolute(&raw_path);
            let src_dir = if project_root.join("src").exists() {
                project_root.join("src")
            } else {
                project_root.clone()
            };

            let build_root = project_root.join("build");
            let log_root = project_root.join("logs");

            let exclude_patterns = vec!["legacy", "templates", "tmp", "temp"];

            // Find target files ending with '_main.tex' or named 'main.tex'
            let tex_files = find_target_tex_files(&src_dir, &exclude_patterns);

            if tex_files.is_empty() {
                println!(" [Warn] No target LaTeX files ('main.tex' or '*_main.tex') found in '{}'", src_dir.display());
                return;
            }

            // Create compilation jobs for each target file & active theme
            let active_themes = theme.active_themes();
            let mut jobs = Vec::new();

            for tex_file in &tex_files {
                for &t in &active_themes {
                    jobs.push(CompileJob {
                        tex_file: tex_file.clone(),
                        theme: t,
                    });
                }
            }

            println!(" [Info] Found {} file(s) x {} theme(s) = {} job(s) to compile:", 
                tex_files.len(), active_themes.len(), jobs.len());
            for j in &jobs {
                println!("   - [{}] {}", j.theme, j.tex_file.display());
            }

            let num_cpus = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1);
            let max_workers = std::cmp::min(8, num_cpus);

            println!("\n [Info] Compiling with {} parallel workers...\n", max_workers);

            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(max_workers)
                .build()
                .expect("Failed to build thread pool");

            let (successes, failures): (Vec<CompileJob>, Vec<CompileJob>) = pool.install(|| {
                jobs.into_par_iter()
                    .partition(|job| compile_latex(job, &src_dir, &build_root, &log_root))
            });

            println!("\n===== Compilation Summary =====");
            if !successes.is_empty() {
                println!(" [Info] Successfully compiled ({}):", successes.len());
                for j in &successes {
                    let stem = j.tex_file.file_stem().unwrap().to_str().unwrap();
                    println!("    ✓ {}_{}.pdf", j.theme, stem);
                }
            }

            if !failures.is_empty() {
                println!("\n[Error] Failed to compile ({}):", failures.len());
                for j in &failures {
                    let stem = j.tex_file.file_stem().unwrap().to_str().unwrap();
                    println!("    ✗ {}_{}.pdf", j.theme, stem);
                }
                std::process::exit(1);
            } else {
                println!("\n [Info] All documents compiled successfully!");
            }
        }
    }
}

fn update_templates(templates_dir: &Path) {
    let git_dir = templates_dir.join(".git");

    if git_dir.exists() {
        println!(" [Info] Pulling latest templates into {}...", templates_dir.display());
        let status = Command::new("git")
            .arg("-C")
            .arg(templates_dir)
            .arg("pull")
            .status();

        match status {
            Ok(s) if s.success() => println!(" [Info] Templates updated successfully!"),
            _ => eprintln!("[Error] Failed to pull updates from template repository."),
        }
    } else {
        println!(" [Info] Cloning templates to {}...", templates_dir.display());
        if let Some(parent) = templates_dir.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let status = Command::new("git")
            .arg("clone")
            .arg(TEMPLATE_REPO_URL)
            .arg(templates_dir)
            .status();

        match status {
            Ok(s) if s.success() => println!(" [Info] Templates downloaded successfully!"),
            _ => eprintln!("[Error] Failed to clone template repository."),
        }
    }
}

fn update_self() {
    println!(" [Info] Re-installing latest latex_handler binary...");

    let update_cmd = format!("curl -sSL {} | bash", INSTALL_SCRIPT_URL);
    let status = Command::new("sh")
        .arg("-c")
        .arg(&update_cmd)
        .status();

    match status {
        Ok(s) if s.success() => println!(" [Info] Self-update completed successfully!"),
        _ => eprintln!("[Error] Self-update failed. Check your network connection."),
    }
}

fn find_target_tex_files(root: &Path, exclude_patterns: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !root.exists() {
        return files;
    }

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name == "main.tex" || file_name.ends_with("_main.tex") {
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

fn mirror_under(root_dir: &Path, src_dir: &Path, tex_file: &Path) -> io::Result<PathBuf> {
    let parent = tex_file.parent().unwrap_or(src_dir);
    let rel = parent.strip_prefix(src_dir).unwrap_or(Path::new(""));
    let target = root_dir.join(rel);
    fs::create_dir_all(&target)?;
    Ok(target)
}

fn compile_latex(
    job: &CompileJob,
    src_dir: &Path,
    build_root: &Path,
    log_root: &Path,
) -> bool {
    let stem = match job.tex_file.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return false,
    };

    let job_name = format!("{}_{}", job.theme, stem);

    let build_dir = match mirror_under(build_root, src_dir, &job.tex_file) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let log_dir = match mirror_under(log_root, src_dir, &job.tex_file) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let pdf_dir = job.tex_file.parent().unwrap_or(src_dir).join("_pdf");
    if fs::create_dir_all(&pdf_dir).is_err() {
        return false;
    }

    let tex_filename = match job.tex_file.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return false,
    };

    let outdir_arg = format!("-outdir={}", build_dir.display());
    let jobname_arg = format!("-jobname={}", job_name);
    let pretex_arg = format!(r#"-pretex=\def\THEME{{{}}}"#, job.theme);

    let output = Command::new("latexmk")
        .arg("-pdf")
        .arg("-g")           // Force la recompilation sans se fier au cache latexmk
        .arg("-usepretex")  // Obligatoire : indique à latexmk d'utiliser la valeur de -pretex !
        .arg(&pretex_arg)
        .arg("-synctex=1")
        .arg("-shell-escape")
        .arg("-interaction=nonstopmode")
        .arg("-halt-on-error")
        .arg(&outdir_arg)
        .arg(&jobname_arg)
        .arg(tex_filename)
        .current_dir(job.tex_file.parent().unwrap_or_else(|| Path::new(".")))
        .output();

    let (success, stderr_bytes) = match output {
        Ok(out) => (out.status.success(), out.stderr),
        Err(_) => (false, Vec::new()),
    };

    // 1. Copie du PDF vers _pdf/<job_name>.pdf
    let pdf_src = build_dir.join(format!("{}.pdf", job_name));
    if pdf_src.exists() {
        let _ = fs::copy(&pdf_src, pdf_dir.join(format!("{}.pdf", job_name)));
    }

    // 2. Copie du SyncTeX vers _pdf/<job_name>.synctex.gz
    let synctex_src = build_dir.join(format!("{}.synctex.gz", job_name));
    if synctex_src.exists() {
        let _ = fs::copy(&synctex_src, pdf_dir.join(format!("{}.synctex.gz", job_name)));
    }

    // 3. Copie/déplacement du log
    let log_src = build_dir.join(format!("{}.log", job_name));
    if log_src.exists() {
        let dest_log = log_dir.join(format!("{}.log", job_name));
        let _ = fs::rename(&log_src, &dest_log).or_else(|_| {
            fs::copy(&log_src, &dest_log).and_then(|_| fs::remove_file(&log_src))
        });
    } else if success || !!stderr_bytes.is_empty() {
        let dest_log = log_dir.join(format!("{}.log", job_name));
        let _ = fs::write(dest_log, stderr_bytes);
    }

    success
}

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