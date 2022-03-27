# JAC (Json API Client)
JAC is a generic json API client.
It'll make the desire custom requests based on the `Config.toml` configuration file.

## Config.toml
This is the JAC's configuration file, here you'll discribe what API's JAC will query.

* Example:
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
client_id = "Client ID"
client_secret = "Client SECRET" # Due the lack of a intermediate(private) server (maybe in the future...) containing the client_secret
```

**NOTE**: You can also have multiple services in a single file

## Information display

* Example
```
Some API's result:
filter_name = "some_value"
```
