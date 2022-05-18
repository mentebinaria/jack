use std::{io::{self, Write}, fs, str::FromStr, collections::HashMap};
use serde::{Serialize, Deserialize};
use toml::value::Table as TomlTable;

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
    filters: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Json,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_ref() {
            "json" => Ok(Self::Json),
            _ => Err("`{s}` is an invalid output format".to_string()),
        }
    }
}

pub struct Services(Vec<Service>);

impl Output {
    fn new(service_name: String) -> Self {
        Self {
            service_name,
            filters: HashMap::new(),
        }
    }

    fn display<P: AsRef<std::path::Path>>(&self, format: Option<OutputFormat>, dest: Option<P>) {
        if let Some(format) = format {
            
            match format {
                OutputFormat::Json => {
                    let content = serde_json::to_string(&self).unwrap();

                    if let Some(dest) = dest {
                        let dest = dest.as_ref();
                        if !dest.exists() {
                            fs::create_dir(dest).unwrap();
                        }

                        let file_name = dest.join(self.service_name.clone() + ".json");
                        let mut file = fs::File::create(file_name).unwrap();
                        file.write_all(content.as_bytes()).unwrap(); 
                    }

                    println!("{content}")

                },
            }
        } else {
            println!("service_name = {:?}", self.service_name);
            self.filters.iter().for_each(|(k, v)| {
                println!("{k} = {v}");
            });
            println!();
        }
    }
}

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

    fn execute(self) -> Output {
        let url = self.generate_url();
        let mut client = smolhttp::Client::new(&url).unwrap();
        let mut headers = vec![];
        let token = self.authenticate();

        println!("service_name = {:?}", self.service_name);

        match (self.headers, token) {
            (Some(h),  _) => {
                h.into_iter().for_each(|(k, v)| {
                    headers.push((k, v.as_str().unwrap().to_owned()))
                });
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

        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        let mut output = Output::new(self.service_name.clone());

        if let Some(filter) = self.filter {
            filter.into_iter().for_each(|(name, value)| {
                output.filters.insert(name.clone(), json.pointer(value.as_str().unwrap())
                .unwrap_or_else(|| {
                    panic!("({name:?}, ({value:?})) trigged an error");
                }).to_string());
            });
        } else {
            json.as_object().unwrap().iter().for_each(|(k, v)| {
                output.filters.insert(k.to_string(), v.to_string());
            });
        }
        
        output
    }

    fn authenticate(&self) -> Option<String> {
        self.oauth.as_ref().map(super::oauth2::authenticate)
    }
}

impl Services {
    pub fn new<P: AsRef<std::path::Path>>(p: P) -> Result<Self, io::Error> {
        let parsed: toml::Value = toml::from_str(&fs::read_to_string(p)?).unwrap();
        let mut services = vec![];

        for (_, values) in parsed.as_table().unwrap() {
            let service: Service = toml::from_str(&toml::to_string(values).unwrap()).unwrap();
            services.push(service);
        }

        Ok(Self(services))
    }

    pub fn statistics<P: AsRef<std::path::Path>>(self, format: Option<OutputFormat>, dest: Option<&P>) {
        self.0.into_iter().for_each(|service| {
            service.execute().display(format, dest)
        });
    }
}