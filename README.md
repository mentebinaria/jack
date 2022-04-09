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

An example configuration file for getting stats from YouTube could be the following:

```toml
[youtube]
name = "Youtube"
url = "https://www.googleapis.com/youtube/v3/channels/"
method = "GET"

[youtube.filter]
title = "/items/0/snippet/title"
contentDetails = "/items/0/contentDetails"
totalViews = "/items/0/statistics/viewCount"

[youtube.params]
part = "snippet,contentDetails,statistics"
id = "UCuQ8zW9VmVyml7KytSqJDzg"
key = "<KEY>"
maxResults = "50"
```

In the above configuration file, `id` is an YouTube Channel ID, which we set to [Papo Binário](https://www.youtube.com/c/papobinario) as an example. In `key` you should put your API key (?).

This would produce the following output:

```
Youtube's result:
contentDetails = {"relatedPlaylists":{"likes":"","uploads":"UUuQ8zW9VmVyml7KytSqJDzg"}}
title = "Papo Binário"
totalViews = "2083346"
```

JACK also supports the YouTube Analytics API. Here's an example:

```toml
[youtube_analytics]
name = "Youtube Analytics"
url = "https://youtubeanalytics.googleapis.com/v2/reports"
method = "GET"

[youtube_analytics.filter]
views = "/rows/0/1"
likes = "/rows/0/2"
subscribersGained = "/rows/0/3"
estimatedMinutesWatched = "/rows/0/0"

[youtube_analytics.oauth]
auth_uri = "https://accounts.google.com/o/oauth2/auth"
token_uri = "https://oauth2.googleapis.com/token"
client_id = "<YOUR CLIENT ID>"
client_secret = "<YOUR CLIENT SECRET>"

[youtube_analytics.params]
ids = "channel==MINE"
metrics = "estimatedMinutesWatched,views,likes,subscribersGained"
startDate = "2017-01-01"
endDate = "2017-12-31"
```

This would produce the following output:

```
Youtube Analytics's result:
estimatedMinutesWatched = 1540589
likes = 32025
subscribersGained = 7585
views = 329717
```

## Running

After you are finished with the configuration, you should be ready to run JACK as easy as:

    ./target/debug/jack
    


