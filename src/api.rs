use std::{io::{self, Write}, fs, str::FromStr, collections::HashMap};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Service {
    service_name: String,
    url: String,
    method: String,
    oauth: Option<toml::value::Table>,
    filter: Option<toml::value::Table>,
    params: Option<toml::value::Table>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Output {
    service_name: String,
    filters: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormats {
    Json,
}

impl FromStr for OutputFormats {
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

    pub fn execute(self, format: Option<OutputFormats>) {
        let url = self.generate_url();
        let token = self.authenticate();

        let content = match self.method.as_ref() {
            "GET" => {
                if let Some(token) = token {
                    smolhttp::Client::new(&url)
                        .unwrap()
                        .get()
                        .headers(vec![("Authorization".to_owned(), format!("Bearer {token}"))])
                        .send()
                        .unwrap()
                        .text()
                } else {
                    smolhttp::get(&url).unwrap().text()
                }
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

        if let Some(format) = format {
            let file_name = output.service_name.clone() + ".json";
            let mut file = fs::File::create(file_name).unwrap();

            match format {
                OutputFormats::Json => {
                    let content = serde_json::to_string(&output).unwrap();
                    file.write_all(content.as_bytes()).unwrap();
                },
            }
        } else {
            println!("service_name = {:?}", output.service_name);
            output.filters.iter().for_each(|(k, v)| {
                println!("{k} = {v}");
            });
            println!();
        }
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

    pub fn statistics(self, format: Option<OutputFormats>) {
        for service in self.0 {
            service.execute(format);
        }
    }
}