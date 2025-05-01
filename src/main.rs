use chrono::Datelike;
use chrono::Local;
use clap::Parser;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::collections::HashMap;
use std::fmt::Debug;
use std::option::Option;
use std::str::FromStr;
use std::vec::Vec;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, required=true)]
    pieces: String,

    #[arg(short, long, required=true)]
    time_class: String,

    #[arg(short, long, required=true)]
    username: String,

    #[arg(long, default_value=None)]
    start_month: Option<u32>,

    #[arg(long, default_value=None)]
    start_year: Option<i32>,

    #[arg(long, default_value=None)]
    end_month: Option<u32>,

    #[arg(long, default_value=None)]
    end_year: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
#[repr(transparent)]
struct GameResult {
    map: HashMap<String, u32>
}

impl GameResult {
    fn merge(&mut self, other: &Self) {
        for (key, value) in other.map.iter() {
            *self.map.entry(key.to_string()).or_default() += value;
        }
    }
}

#[derive(Clone, Serialize)]
struct Entry {
    count: u32,
    eco_url: String,
    eco: String,
    results: GameResult,
    average_accuracy: Option<f32>,
}

impl Entry {
    fn merge(&mut self, other: &Self) {
        self.count += other.count;
        if self.eco_url.is_empty() {
            self.eco_url = other.eco_url.to_owned();
        }
        if self.eco.is_empty() {
            self.eco = other.eco.to_owned();
        }
        self.results.merge(&other.results);
        match (self.average_accuracy, other.average_accuracy) {
            (Some(a_avg), Some(b_avg)) => Some((a_avg + b_avg) / 2.0),
            (Some(a_avg), None) => Some(a_avg),
            (None, Some(b_avg)) => Some(b_avg),
            (None, None) => None
        };
    }
}

#[derive(Clone, Deserialize)]
struct Accuracies {
    black: f32,
    white: f32,
}

#[derive(Clone, Deserialize)]
struct PieceInfo {
    result: String,
}

#[derive(Clone, Deserialize)]
struct Game {
    accuracies: Option<Accuracies>,
    pgn: String,
    black: PieceInfo,
    white: PieceInfo,
    time_class: String,
}

#[derive(Deserialize)]
struct ResponseJson {
    games: Vec<Game>
}

type OutputJson = HashMap<String, Entry>;

#[derive(Serialize)]
struct Output {
    json: OutputJson
}

struct ParsedPgnTags {
    black: String,
    white: String,
    eco: String,
    eco_url: String,
}

struct ParsedPgnTagsError {
    message: String
}

impl FromStr for ParsedPgnTags {
    type Err = ParsedPgnTagsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let error_message: String = String::from("Could not parse tags");
        let err = ParsedPgnTagsError { message: error_message };

        fn get_match(s: &str, key: &str) -> Option<String> {
            match s.find(&format!("{} ", key)) {
                Some(m) => {
                    let start_index = m + key.len() + 2; // ' "'
                    for (i, c) in s[start_index..].chars().enumerate() {
                       if c != ']' {
                           continue;
                       }

                       let end_index = start_index + i - 1; // ignore end quote
                       let value = s[start_index..end_index].to_owned();
                       return Some(value);
                    }

                    None
                },
                None => None,
            }
       }

       let black: String = match get_match(s, "Black") {
           Some(m) => m,
           None => return Err(err),
       };
       let white: String = match get_match(s, "White") {
           Some(m) => m,
           None => return Err(err),
       };
       let eco: String = match get_match(s, "ECO") {
           Some(m) => m,
           None => return Err(err),
       };
       let eco_url: String = match get_match(s, "ECOUrl") {
           Some(m) => m,
           None => return Err(err)
       };

       Ok(ParsedPgnTags { black, white, eco, eco_url })
   }
}

fn get_game_accuracy(user_pieces: &str, game: &Game) -> Option<f32> {
    match game.accuracies {
        Some(ref accuracies) => {
            if user_pieces == "Black" {
                Some(accuracies.black)
            } else {
                Some(accuracies.white)
            }
        },
        None => None
    }
}

fn get_user_pieces<'cli>(cli: &'cli Cli, tags: &ParsedPgnTags) -> Option<&'cli str> {
    let black = tags.black.to_owned();
    let white = tags.white.to_owned();

    if black.to_lowercase() == cli.username.to_lowercase() {
        return Some("Black");
    }

    if white.to_lowercase() == cli.username.to_lowercase() {
        return Some("White");
    }

    None
}

fn get_opening_name(tags: &ParsedPgnTags) -> Option<&str> {
    let eco_url: &String = &tags.eco_url;
    let parts: Vec<&str> = eco_url.split('/').collect();
    let name = parts[parts.len() - 1];
    Some(name)
}

fn get_game_result(user_pieces: String, game: &Game) -> GameResult {
    let result: String = if user_pieces == "Black" {
        game.black.result.to_owned()
    } else {
        game.white.result.to_owned()
    };

    let mut results = GameResult { map: HashMap::new() };
    results.map.insert(result, 1);
    results
}

fn add_game_to_output(cli: &Cli, json: &mut OutputJson, game: &Game) {
    let tags: ParsedPgnTags = match game.pgn.parse::<ParsedPgnTags>() {
        Ok(tags) => tags,
        Err(e) => { eprintln!("{}", e.message); return }
    };

    let user_pieces = match get_user_pieces(cli, &tags) {
        Some(user_pieces) => user_pieces,
        None => return,
    };
    if user_pieces.to_lowercase() != cli.pieces.to_lowercase() {
        return;
    }

    let time_class = &game.time_class;
    if time_class.to_lowercase() != cli.time_class.to_lowercase() {
        return;
    }

    let accuracy: Option<f32> = get_game_accuracy(user_pieces, game);

    let opening_name = match get_opening_name(&tags) {
        Some(opening_name) => opening_name,
        None => return,
    };

    let results: GameResult = get_game_result(user_pieces.to_string(), game);

    let mut new_entry = Entry {
        average_accuracy: accuracy,
        count: 1,
        eco: tags.eco.to_owned(),
        eco_url: tags.eco_url.to_owned(),
        results,
    };

    if let Some(existing_entry) = json.get(opening_name) {
        new_entry.merge(existing_entry)
    }

    json.insert(opening_name.to_string(), new_entry);
}

async fn get_month_games(cli: &Cli, client: &reqwest::Client, month: u32, year: i32) -> Option<ResponseJson> {
    let now = Local::now();
    let current_month = now.date_naive().month();
    let current_year = now.date_naive().year();

    if current_year == year && month > current_month {
        return None
    }

    let url = format!(
        "https://api.chess.com/pub/player/{}/games/{}/{:02}",
        cli.username,
        year,
        month
    );

    let result = client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36"
        )
        .send()
        .await;

    let response = match result {
        Ok(response) => Some(response),
        Err(e) => {
            eprintln!("{}", e);
            None
        },
    };

    match response {
        Some(response) => {
            let status = response.status().to_owned();
            let result = response.json::<ResponseJson>().await;
            match result {
                Ok(response) => Some(response),
                Err(e) => {
                    eprintln!("{} ({}):  {}", &url, status, e);
                    None
                },
            }
        },
        None => None,
    }
}

async fn get_games(cli: &Cli, client: &reqwest::Client) -> Vec<Option<ResponseJson>> {
    let now = Local::now();
    let year = now.date_naive().year();

    let start_month = cli.start_month.unwrap_or(1);
    let end_month = match cli.end_month {
        Some(m) => m + 1,
        None => 13
    };

    let start_year = cli.start_year.unwrap_or(year);
    let end_year = match cli.end_year {
        Some(y) => y + 1,
        None => year + 1,
    };

    let mut response_futures: Vec<_> = vec![];
    for y in start_year..end_year {
        for m in start_month..end_month {
            response_futures.push(get_month_games(cli, client, m, y));
        }
    }

    join_all(response_futures).await
}

async fn run(cli: &Cli) {
    let client = match reqwest::Client::builder().build() {
        Ok(c) => c,
        Err(e) => { eprintln!("{}", e); std::process::exit(1); }
    };
    let responses = get_games(cli, &client).await;

    let mut game_count = 0;
    let mut json: OutputJson = HashMap::new();
    for response in responses {
        match response {
            None => continue,
            Some(r) => {
                game_count += r.games.len();
                for game in r.games {
                    add_game_to_output(cli, &mut json, &game);
                }
            },
        }
    }
    eprintln!("Parsed {} Games", game_count);

    let output = Output {
        json
    };

    let output_string = serde_json::to_string(&output.json);
    match output_string {
        Ok(str) => println!("{}", str),
        Err(e) => eprintln!("{}", e),
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    run(&cli).await;
}
