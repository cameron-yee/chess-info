# Chess Info

CLI tool for analyzing games from [chess.com](https://www.chess.com/) using the [chess.com API](https://www.chess.com/clubs/forum/view/guide-unofficial-api-documentation).

## Installation

```sh
cargo install --git https://github.com/cameron-yee/chess-info
```

## Usage

```sh
Usage: chess-info [OPTIONS] --time-class <TIME_CLASS> --username <USERNAME>

Options:
  -p, --pieces <PIECES>
  -t, --time-class <TIME_CLASS>
  -u, --username <USERNAME>
      --start-month <START_MONTH>
      --start-year <START_YEAR>
      --end-month <END_MONTH>
      --end-year <END_YEAR>
  -b, --best-opening
  -h, --help                       Print help
  -V, --version                    Print version
```

### Example

```sh
chess-info --pieces black --time-class rapid --username cameron-yee --start-year 2025
```

## Data Flow

1. Take options from input (CLI)
2. Fetch all user games (or just games by year or month depending on flags)
   - https://api.chess.com/pub/player/{username}/games/archives
   - https://api.chess.com/pub/player/{username}/games/{year}/{month}
3. Filter games
  - game.time_class (bullet | blitz | rapid | daily)
4. Map all games
  - game.accuracies
  - game.pgn
    - https://docs.rs/pgn-reader/latest/pgn_reader/struct.BufferedReader.html
    - https://github.com/mliebelt/pgn-parser/tree/main
    - piece color for user (Black | White)
    - ECO Name (parse ECOUrl)
5. Print formatted results to stdout
  - opening URL (pgn.ECOUrl)
  - total games by opening
  - average accuracy by opening
  - results by opening

## Resources

- https://stackoverflow.com/questions/51044467/how-can-i-perform-parallel-asynchronous-http-get-requests-with-reqwest

## Development

### Running local changes

```sh
cargo build --release
./target/release/chess-info --pieces black --username cameron-yee --start-year 2025 --start-month 1 --end-month 2 --time-class blitz
```

### Profiling for bottlenecks

```sh
CARGO_PROFILE_RELEASE_DEBUG=true sudo flamegraph -- ./target/release/chess-info --pieces black --username hikaru --start-year 2020 --time-class blitz
```
