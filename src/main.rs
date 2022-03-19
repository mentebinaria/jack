mod api;
mod oauth2;

fn main() {
    let services = api::Services::new().unwrap();
    services.statistics();
}
