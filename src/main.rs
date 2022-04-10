use std::{str::FromStr, collections::HashMap};

mod api;
mod oauth2;

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
enum Argument {
    Config,
    Format,
    Dest,
}

impl FromStr for Argument {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_ref() {
            "--config" | "-c" => Ok(Argument::Config),
            "--format" | "-f" => Ok(Argument::Format),
            "--dest" => Ok(Argument::Dest),
            _ => Err("Invalid option".to_string()),
        }
    }
}

fn main() {
    let default_config_file = Some("config.toml".to_string());

    let mut args = std::env::args().skip(1);
    let mut opts: HashMap<Argument, Option<String>> = HashMap::new();

    while let Some(arg) = args.next() {
        if arg.starts_with("--") {
            let argument = Argument::from_str(&arg).unwrap();
            opts.insert(argument, args.next());
        }
    }

    let config_file = opts.get(&Argument::Config)
        .unwrap_or(&default_config_file).as_ref();

    let services = api::Services::new(config_file.unwrap()).unwrap();
    
    let output_format = opts.get(&Argument::Format)
        .map(|fmt| api::OutputFormat::from_str(fmt.as_ref().unwrap()).unwrap());

    let dest = opts.get(&Argument::Dest).map(|path| path.as_ref().unwrap());
    
    services.statistics(output_format, dest);

}
