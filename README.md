# JACK (JSON API Client Konsumer)

JACK is a generic JSON API client. It is useful to interact with APIs from multiple services such as Google and Twitter. All service along with their parameters are configured in `config.toml` file. For example, you can configure JACK to collect statistics from vairous social networks and Google Analytics, all from a single company/instituition.

## Installing

JACK is written in Rust, so you need to install it first before compiling. Here's a few examples on how to install the Rust toolset:

### Linux

Different Linux flavors provide different package managers. Here are few examples:

    sudo apt install rust
    sudo yum install rust
   
### macOS

First, install Homebrew by following the instructions [here](https://brew.sh). Then, open a terminal and use the `brew` command to install the Rust toolset:
    
    brew install rust
    
### Windows

Windows 11 comes with `winget`, a command-line package manager. You can open a Windows Terminal and type the following command:

    winget install Rustlang.Rust.MSVC
    
If you don't have `winget`, you can either [install it](https://docs.microsoft.com/en-us/windows/package-manager/winget/) or download and install Rust from its the [official website](https://www.rust-lang.org/tools/install).
    
After Rust is intalled, clone JACK's repository and build it:    

    git clone https://github.com/mentebinaria/jack/
    cd jack
    cargo build
    
## Configuration

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

### YouTube API

Let's say you want some information about a certain YouTube channel. Start with creating a service in the `config.toml` file. Give preference to single words, but `snake_case` is allowed. As this service is a YouTube channel, you use the YouTube API in the `url` field:

```toml
[mychannel]
name = "Great YouTube Channel - YouTube Example"
url = "https://www.googleapis.com/youtube/v3/channels/"
method = "GET"
```

Now you have to tell JACK what kind of information you want to get from this channel. Available fields depend on the API. For YouTube API, they are documented [here](https://developers.google.com/youtube/v3/docs/channels). Let's say we want `title`, `contentDetails` and `statistics/viewCount`. Here's how to configure them:

```toml
[mychannel.filter]
title = "/items/0/snippet/title"
contentDetails = "/items/0/contentDetails"
totalViews = "/items/0/statistics/viewCount"
```

Now you have to configure the parameters such as the desired YouTube channel ID you want to inspect, your [API key](https://developers.google.com/youtube/v3/getting-started) and a few others. Here's an example with [Papo Binário](https://www.youtube.com/c/papobinario) YouTube channel, whose ID is `UCuQ8zW9VmVyml7KytSqJDzg`:

```toml
[mychannel.params]
part = "snippet,contentDetails,statistics"
id = "UCuQ8zW9VmVyml7KytSqJDzg"
key = "<KEY>"
maxResults = "50"
```

Done. When you run JACK with the above configuration, you should see the following output:

```
name = "Great YouTube Channel - YouTube Example"
contentDetails = {"relatedPlaylists":{"likes":"","uploads":"UUuQ8zW9VmVyml7KytSqJDzg"}}
title = "Papo Binário"
totalViews = "2083346"
```

### YouTube Analytics API

For advanced metrics on YouTube, they offer separate service called YouTube Analytics. JACK can also interact with its API and perform OAuth authentication. Here's an example of configuration for getting the number of views, likes, subscribers gained, and the channel estimate minutes watched in January, 2021:

```toml
[mychannel_analytics]
name = "My Great Channel - YouTube Analytics Example"
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
startDate = "2021-01-01"
endDate = "2021-02-01"
```
Before running JACK with the above configuration, you need to provide a `client_id` and a `client_secret` for the YouTube Analytics API. Refer to [this article](https://developers.google.com/youtube/registering_an_application) if you don't have them yet.

JACK's output would be like the following:

```
name = "My Great Channel - YouTube Analytics Example"
estimatedMinutesWatched = 162626
likes = 2964
subscribersGained = 646
views = 37745
```

If you study the services' API enough, you'll be ready to use JACK to extract every possible bit of information you need. :)

## Running

After you are finished with the configuration, you should be ready to run JACK as easy as:

    cargo run
    
