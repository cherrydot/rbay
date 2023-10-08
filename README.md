# Pirate Bay Client

This is a minimal wrapper for [The Pirate Bay](https://thepiratebay.org/). It provides
functions to search for and fetch metadata on torrents. It is *not* a torrenting client:
it only provides metadata. At present it does not support uploading torrents or managing
user accounts either.

See the docs for more information including example usage.

## Development

The data in `src/scraped.rs` is generated by `scraper.py`, which can be run as follows:

```
$ python3 scraper.py -c > src/scraped.rs
```
