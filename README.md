## Data Flow

1. Take options from input (CLI)
2. Fetch all user games (or just games by year or month depending on flag)
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
