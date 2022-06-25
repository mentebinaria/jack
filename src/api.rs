use std::{
    io::{self, Write},
    fs, str::FromStr,
    collections::HashMap,
    path::{Path, PathBuf}
};
use serde::{Serialize, Deserialize};
use toml::value::Table as TomlTable;
use serde_json::Value as JsonValue;

// Type Aliases
type OAuth = Option<TomlTable>;
type Filter = Option<TomlTable>;
type Params = Option<TomlTable>;
type Headers = Option<TomlTable>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Service {
    service_name: String,
    url: String,
    method: String,
    oauth: OAuth,
    filter: Filter,
    params: Params,
    headers: Headers,
}

#[derive(Serialize, Deserialize, Debug)]
struct Output {
    service_name: String,
    output_format: OutputFormat,
    dest: Option<PathBuf>,
    filters: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum OutputFormat {
    Json,
    Pretty,
    PrettyJson,
}


impl Output {
    fn new<P: AsRef<Path>>(service_name: String, output_format: OutputFormat, dest: Option<&P>) -> Self {
        Self {
            service_name,
            output_format,
            dest: dest.map(|dest| dest.as_ref().to_path_buf()),
            filters: HashMap::new(),
        }
    }
}

pub type Services = Vec<Service>;

impl Service {
    fn generate_url(&self) -> String {
        let mut url = self.url.clone();
        
        match self.method.as_ref() {
            "GET" => {
                if let Some(params) = self.params.as_ref() {
                    url.push('?');
                    for (key, value) in params {
                        let value = value.as_str().unwrap();            
                        url += &format!("{key}={value}&");
                    }
                }

                if url.ends_with('&') {
                    url.pop();
                }

                url
            },
            "POST" => {
                println!("Can't use POST request for {:?} target", self.service_name);
                String::new()
            },
            _ => String::new()
        }
    }

    fn execute<P: AsRef<Path>>(self, output_format: OutputFormat, dest: Option<&P>) -> Output {
        let url = self.generate_url();
        let mut client = smolhttp::Client::new(&url).unwrap();
        let mut headers = vec![];
        let token = self.authenticate();

        match (self.headers, token) {
            (Some(header),  _) => {
                headers.extend(header.into_iter().map(|(k, v)| (k, v.as_str().unwrap().to_owned())));
            },
            (_, Some(token)) => {
                headers.push(("Authorization".to_owned(), format!("Bearer {token}")));
            }
            _ => ()
        }

        client.headers(headers);

        let content = match self.method.as_ref() {
            "GET" => {
                client.get().send().unwrap_or_else(|err| {
                    eprintln!("Could not complete the request, try delete the `.oauth_tokens` file an retry\nError: {err}");
                    std::process::exit(1);
                }).text()
            },
            _ => panic!("No support for {:?} requests", self.method),
        };

        let json: JsonValue = serde_json::from_str(&content).unwrap();
        let mut output = Output::new(self.service_name.clone(), output_format, dest);

        if let Some(filter) = self.filter {
            for (name, value) in filter {
                output.filters.insert(name.clone(), json.pointer(value.as_str().unwrap())
                .unwrap_or_else(|| {
                    panic!("({name:?}, ({value:?})) trigged an error");
                }).to_string());
            }
        } else {
            for (k, v) in json.as_object().unwrap() {
                output.filters.insert(k.to_string(), v.to_string());
            }
        }
        
        output
    }

    fn authenticate(&self) -> Option<String> {
        if let Some(oauth) = &self.oauth {
            super::oauth2::authenticate(oauth, &self.service_name).ok()
        } else {
            None
        }
    }
}

pub fn parse<P: AsRef<Path>>(p: &P) -> Result<Services, io::Error> {
    let parsed: toml::Value = toml::from_str(&fs::read_to_string(p)?).unwrap();
    let mut services = vec![];

    for (_, values) in parsed.as_table().unwrap() {
        let service: Service = toml::from_str(&toml::to_string(values).unwrap()).unwrap();
        services.push(service);
    }

    Ok(services)
}

pub fn run(services: Services, format: OutputFormat, dest: Option<PathBuf>) {
    for service in services {
        println!("{}", service.execute(format, dest.as_ref()));
    }
}

// Trait impl's

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_ref() {
            "json" => Ok(Self::Json),
            "pretty" => Ok(Self::Pretty),
            "prettyjson" | "pjson" => Ok(Self::PrettyJson),
            _ => Err("`{s}` is an invalid output format".to_string()),
        }
    }
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.output_format {
            OutputFormat::Json => {
                let content = serde_json::to_string(&self).unwrap();

                if let Some(dest) = self.dest.as_ref() {
                    if !dest.exists() {
                        fs::create_dir(dest).unwrap();
                    }
        
                    let file_name = dest.join(self.service_name.clone() + ".json");
                    let mut file = fs::File::create(file_name).unwrap();
                    file.write_all(content.as_bytes()).unwrap(); 
                }

                writeln!(f, "{content}")?;
            },
            OutputFormat::Pretty => {
                writeln!(f, "service_name = {:?}", self.service_name)?;
                for (k, v) in self.filters.iter() {
                    writeln!(f, "{k} = {v}")?;
                }
                writeln!(f)?;
            },
            OutputFormat::PrettyJson => {
                let content = serde_json::to_string_pretty(&self).unwrap();

                if let Some(dest) = self.dest.as_ref() {
                    if !dest.exists() {
                        fs::create_dir(dest).unwrap();
                    }
        
                    let file_name = dest.join(self.service_name.clone() + ".json");
                    let mut file = fs::File::create(file_name).unwrap();
                    file.write_all(content.as_bytes()).unwrap(); 
                }

                writeln!(f, "{content}")?;
            }
        }

        Ok(())
    }
}