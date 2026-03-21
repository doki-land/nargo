use crate::{config::Config, generator::Generator, server::DevServer};
use clap::{Arg, Command};
use std::path::Path;

mod config;
mod generator;
mod plugin;
mod server;
mod templates;
mod theme;

fn main() {
    let matches = Command::new("nargo-document").version("0.1.0").about("HXO Document Generator").subcommand(Command::new("build").about("Build static documentation").arg(Arg::new("config").short('c').long("config").help("Path to config file").default_value("nargodoc.config.toml")).arg(Arg::new("outDir").short('o').long("outDir").help("Output directory").default_value("dist"))).subcommand(Command::new("dev").about("Start development server").arg(Arg::new("config").short('c').long("config").help("Path to config file").default_value("nargodoc.config.toml")).arg(Arg::new("port").short('p').long("port").help("Server port").default_value("3000"))).subcommand(Command::new("serve").about("Serve built documentation").arg(Arg::new("outDir").short('o').long("outDir").help("Output directory").default_value("dist")).arg(Arg::new("port").short('p').long("port").help("Server port").default_value("3000"))).get_matches();

    match matches.subcommand() {
        Some(("build", sub_matches)) => {
            let config_path = sub_matches.get_one::<String>("config").unwrap();
            let out_dir = sub_matches.get_one::<String>("outDir").unwrap();

            println!("Building documentation...");
            println!("Config: {}", config_path);
            println!("Output: {}", out_dir);

            // Load config
            let config = if Path::new(config_path).exists() {
                match Config::load_from_file(config_path) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("Error loading config: {}", e);
                        Config::default()
                    }
                }
            }
            else {
                println!("Config file not found, using default config");
                Config::default()
            };

            println!("Config loaded: {:?}", config);

            // Create generator and generate documentation
            let mut generator = Generator::new(config);
            if let Err(e) = generator.generate(".", out_dir) {
                eprintln!("Error generating documentation: {}", e);
            }
            else {
                println!("Documentation generated successfully!");
            }
        }
        Some(("dev", sub_matches)) => {
            let config_path = sub_matches.get_one::<String>("config").unwrap();
            let port = sub_matches.get_one::<String>("port").unwrap().parse::<u16>().unwrap();

            println!("Starting development server...");
            println!("Config: {}", config_path);
            println!("Port: {}", port);

            // Load config
            let config = if Path::new(config_path).exists() {
                match Config::load_from_file(config_path) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("Error loading config: {}", e);
                        Config::default()
                    }
                }
            }
            else {
                println!("Config file not found, using default config");
                Config::default()
            };

            // Create dev server and start
            let mut dev_server = DevServer::new(config, port, "dist");
            tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(dev_server.start()).unwrap();
        }
        Some(("serve", sub_matches)) => {
            let out_dir = sub_matches.get_one::<String>("outDir").unwrap();
            let port = sub_matches.get_one::<String>("port").unwrap();

            println!("Serving documentation...");
            println!("Directory: {}", out_dir);
            println!("Port: {}", port);

            // TODO: Implement serve logic
        }
        _ => {
            println!("Please specify a subcommand: build, dev, or serve");
        }
    }
}
