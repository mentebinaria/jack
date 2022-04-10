use std::{str::FromStr, collections::HashMap};

mod api;
mod oauth2;

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
enum Argument {
    Config,
    Format,
}

impl FromStr for Argument {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_ref() {
            "--config" | "-c" => Ok(Argument::Config),
            "--format" | "-f" => Ok(Argument::Format),
            _ => Err("Invalid option".to_string()),
        }
    }
}

fn main() {
    let default_config_file = "config.toml".to_string();

    let mut args = std::env::args().skip(1);
    let mut opts: HashMap<Argument, String> = HashMap::new();

    while let Some(arg) = args.next() {
        if arg.starts_with("--") {
            let argument = Argument::from_str(&arg).unwrap();
            opts.insert(argument, args.next().unwrap());
        }
    }

    let config_file = opts.get(&Argument::Config)
        .unwrap_or(&default_config_file);
    let services = api::Services::new(&config_file).unwrap();
    
    let output_format = opts.get(&Argument::Format)
        .map(|fmt| api::OutputFormats::from_str(fmt).unwrap());
    services.statistics(output_format);

}
