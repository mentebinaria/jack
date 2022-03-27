use std::{io, fs};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Service {
    name: String,
    url: String,
    method: String,
    oauth: Option<toml::value::Table>,
    filter: Option<toml::value::Table>,
    params: Option<toml::value::Table>,
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
                println!("Can't use POST request for {:?} target", self.name);
                String::new()
            },
            _ => String::new()
        }
    }

    pub fn execute(&self) {
        let url = self.generate_url();
        println!("{}'s result:", self.name);
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
        if let Some(filter) = &self.filter {
            filter.iter().for_each(|(name, value)| {
                println!("{name} = {}", json.pointer(value.as_str().unwrap()).unwrap());
            });
        }
        println!()
    }

    fn authenticate(&self) -> Option<String> {
        self.oauth.as_ref().map(super::oauth2::authenticate)
    }
}

impl Services {
    pub fn new() -> Result<Self, io::Error> {
        let parsed: toml::Value = toml::from_str(&fs::read_to_string("Config.toml")?).unwrap();
        let mut services = vec![];

        for (_, values) in parsed.as_table().unwrap() {
            let service: Service = toml::from_str(&toml::to_string(values).unwrap()).unwrap();
            services.push(service);
        }

        Ok(Self(services))
    }

    pub fn statistics(&self) {
        for service in self.0.iter() {
            service.execute();
        }
    }
}