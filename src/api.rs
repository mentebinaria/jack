use std::{io, fs};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Service {
    service_name: String,
    url: String,
    method: String,
    oauth: Option<toml::value::Table>,
    filter: Option<toml::value::Table>,
    params: Option<toml::value::Table>,
    headers: Option<toml::value::Table>,
}

pub struct Services(Vec<Service>);

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

    pub fn execute(self) {
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
        if let Some(filter) = &self.filter {
            filter.iter().for_each(|(name, value)| {
                println!("{name} = {}", json.pointer(value.as_str().unwrap()).unwrap());
            });
        } else {
            println!("{}", json)
        }
        println!();
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

    pub fn statistics(self) {
        for service in self.0 {
            service.execute();
        }
    }
}