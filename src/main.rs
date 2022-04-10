mod api;
mod oauth2;

fn main() {
    let file = std::env::args().nth(1).unwrap_or_else(|| "config.toml".to_string());
    let services = api::Services::new(&file).unwrap();
    services.statistics();
}
