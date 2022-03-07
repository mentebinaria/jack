mod api;

fn main() {
    let services = api::Services::new().unwrap();
    services.statistics();
}
