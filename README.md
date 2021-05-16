# doctoscrape
Scrape doctolib for next-day vaccination appointments

## Build

```sh
cargo build
```

## Test

```sh
cargo test
```

## Run

The only required command-line argument is the postal code that you're searching
around. For example, to find all appointments around the 1st arrondissement, run:

```sh
cargo run -- 75001
```

### Additional options

All extra options can be discovered using `-h` or `--help`.

The tool defaults to searching within `paris`, but the `-c` flag overrides this:

```sh
cargo run -- 14000 -c caen
```

Since the search radius can be wider than expected, especially in a city,
exclude aany postal code with the `-x` flag:

```sh
# reject results in Beauvais 66000
cargo run -- 75001 -x 66000
```

Specify how many search result pages to scrape with the `-p` flag (defaults to `1`):

```sh
cargo run -- 75001 -p 5
```

## Logging

For nice integration with crontab and `MAILTO`, this program uses `env_logger`
to avoid printing too much. For the most part, run this as a cronjob with
`RUST_LOG=doctoscrape=info` to only print out real results. This way, you will
only receive an email on a hit!

## Next steps

* Allow the `-x` flag to be used multiple times
* Test in more cities than just Paris
