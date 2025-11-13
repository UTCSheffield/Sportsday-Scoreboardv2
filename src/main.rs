use clap::Parser;

mod db_configurator;

#[derive(Parser, Debug)]
#[clap(author = "Sean Outra", version, about)]
/// Application configuration
struct Args {
    /// whether to be verbose
    #[arg(short = 'v')]
    verbose: bool,

    /// an optional name to greet
    #[arg()]
    name: Option<String>,

    /// path to YAML config file
    #[arg(short = 'c', long = "config")]
    config_path: Option<String>,
}

fn main() {
    let args = Args::parse();
    if args.verbose {
        println!("DEBUG {args:?}");
    }

    // If config path provided, try to load and display it
    if let Some(config_path) = args.config_path {
        match db_configurator::parser::DBConfiguration::from_yaml_file(&config_path) {
            Ok(config) => {
                println!("Successfully loaded configuration from: {}", config_path);
                println!("Years: {}", config.years.len());
                println!("Forms: {}", config.forms.len());
                println!("Events: {}", config.events.len());

                // Show active years
                let active_years = config.years;
                if !active_years.is_empty() {
                    println!("\nActive years:");
                    for year in active_years {
                        println!("  - {}", year.name);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error loading config: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!(
            "Hello {} (from sportsday-scoreboard-v2)!",
            args.name.unwrap_or("world".to_string())
        );
        println!("Use --config <path> to load a YAML configuration file");
    }
}
