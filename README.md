# JACK (JSON API Client Konsumer)

JACK is a generic JSON API client. It is useful to interact with APIs from multiple services such as Google and Twitter. All service along with their parameters are configured in `config.toml` file. For example, you can configure JACK to collect statistics from vairous social networks and Google Analytics, all from a single company/instituition.

## Compiling

JACK is written in Rust, so you need to install it first before compiling.

    git clone https://github.com/mentebinaria/jack/
    cd jack
    cargo build
    
## Configuring

Before running JACK, you have to configure your `config.toml` file. Here's how it works:

```toml
[some_api]
name = "Some API" # The name of the application
url = "http://someapi.xyz" # The url that it'll query
method = "GET" # The method to use (only GET for now)

[some_api.filter] # Optional
filter_name = "/field/item" # The filter is based on https://docs.serde.rs/serde_json/value/enum.Value.html#method.pointer
# ...

[some_api.params] # Optional
params = "value" # `?params=value` on GET requests
# ...

[some_api.oauth] # Optional
auth_uri = "https://some_api/oauth/auth"
token_uri = "https://some_api/oauth/token"
client_id = "Client ID" # Same reason why as the `client_secret`
client_secret = "Client SECRET" # Due the lack of a intermediate(private) server (maybe in the future...) containing the client_secret
```

## Running

After you are finished with the configuration, you should be ready to run JACK as easy as:

    ./target/debug/jack
    
An example output:

