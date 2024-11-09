use std::{fs, io::ErrorKind, path::Path};
use colored::Colorize;

fn main() {
    let mut args = std::env::args();
    let config = match parse_config(&mut args) {
        Ok(c) => c,
        Err(msg) => {
            error_message(msg.as_str());
            std::process::exit(1);
        }
    };

    match execute_cmd(&config) {
        Ok(msg) => success_message(msg.as_str()),
        Err(msg) => error_message(msg.as_str()),
    };
}

struct Config {
    cmd: String,
    params: Vec<String>,
}

impl Config {
    fn new() -> Config {
        Config {
            cmd: String::new(),
            params: Vec::new(),
        }
    }
}

fn parse_config(args: &mut std::env::Args) -> Result<Config, String> {
    let mut config = Config::new();
    args.next();

    config.cmd = match args.next() {
        Some(s) => s,
        None => return Err("Inproper use. Proper use: nerd <command>.".to_string()),
    };

    for arg in args {
        config.params.push(arg);
    }

    return Ok(config);
}

fn execute_cmd(config: &Config) -> Result<String, String> {
    match config.cmd.as_str() {
        "start" => start_project(&config.params),
        _ => Err(format!("Unknown command: {}", config.cmd)),
    }
}

fn start_project(params: &Vec<String>) -> Result<String, String> {
    let mut params = params.into_iter();
    let project_name = match params.next() {
        Some(s) => s,
        None => return Err("Inproper use. Proper use: nerd start <project_name>.".to_string()),
    };

    info_message("Generating project files...");
    
    let project_dir = Path::new(project_name);
    match fs::create_dir(project_dir.to_path_buf()) {
        Ok(_) => (),
        Err(e) => return Err(format!("Failed to create project dir: {}", match e.kind() {
            ErrorKind::AlreadyExists => format!("Directory '{}' already exists.", project_dir.to_str().unwrap()),
            _ => e.to_string(),
        })),
    }

    fn clean_up(project_dir: &Path) {
        let _ = std::process::Command::new("rm")
            .arg("-rf")
            .arg(project_dir.to_str().unwrap())
            .output();
    }

    let source_dir = project_dir.join("src");
    let build_dir = project_dir.join("build");

    let _ = fs::create_dir(&source_dir); 
    let _ = fs::create_dir(&build_dir);

    let cpp_file_content = "\
        #include <iostream>\n\
        \n\
        int main() {\n\
        \tstd::cout << \"Hello, World!\" << std::endl;\n\
        }".to_string();

    let cmake_file_content = format!("\
        cmake_minimum_required(VERSION 3.5)\n\
        \n\
        set(CMAKE_CXX_STANDARD 20)\n\
        set(CMAKE_CXX_STANDARD_REQUIRED True)\n\
        set(CMAKE_EXPORT_COMPILE_COMMANDS True)\n\
        \n\
        project({} VERSION 0.1)\n\
        \n\
        add_executable(${{PROJECT_NAME}} src/main.cpp)",
        project_name);

    let _ = fs::write(source_dir.join("main.cpp"), cpp_file_content);
    let _ = fs::write(project_dir.join("CMakeLists.txt"), cmake_file_content);

    info_message("Project files generated successfully.");
    info_message("Generating build files...");

    let result = std::process::Command::new("cmake")
        .arg(format!("-S {}", project_dir.to_str().unwrap()))
        .arg(format!("-B {}", build_dir.to_str().unwrap()))
        .output();

    let output = match result {
        Ok(o) => o,
        Err(e) => {
            clean_up(&project_dir);
            return Err(
                format!("Failed to generate build files: {}", match e.kind() {
                    ErrorKind::NotFound => "CMake not found.".to_string(),
                    _ => e.to_string(),
                })
        )},
    };

    if !output.status.success() {
        clean_up(&project_dir);
        return Err(format!("Failed to generate build files: CMake error.\n{}", String::from_utf8(output.stderr).unwrap().trim()));
    }
    
    info_message("Build files generated successfully.");

    Ok(format!("Project '{}' created successfully!", project_name))
}

fn error_message(msg: &str) {
    eprintln!("{}", msg.red());
}

fn info_message(msg: &str) {
    println!("{}", msg);
}

fn success_message(msg: &str) {
    println!("{}", msg.bright_green());
}
