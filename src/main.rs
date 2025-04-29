use chrono::Datelike;
use chrono::Local;
use clap::Parser;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::collections::HashMap;
use std::fmt::Debug;
use std::option::Option;
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

    #[arg(short, long, required=true)]
    year: String,
}

#[derive(Clone, Debug, Serialize)]
struct Entry {
    count: u32,
    results: HashMap<String, u32>,
    average_accuracy: Option<f32>,
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

type GameResult = HashMap<String, u32>;
type ParsedPgnTags<'a> = HashMap<&'a str, String>;

async fn get_month_games(cli: &Cli, client: &reqwest::Client, month: u32) -> Option<ResponseJson> {
    let result = client
        .get(
            format!(
                "https://api.chess.com/pub/player/{}/games/{}/{:02}",
                cli.username,
                cli.year,
                month
            ))
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36"
        )
        .send()
        .await;

    let response = match result {
        Ok(response) => response,
        Err(e) => { println!("{}", e); panic!() },
    };

    let result = response.json::<ResponseJson>().await;
    match result {
        Ok(response) => Some(response),
        Err(e) => {
            println!("{}", e);
            None
        },
    }
}

fn parse_pgn_tags(pgn: &str) -> ParsedPgnTags {
    let re = regex::RegexBuilder::new(r"^\[([^\]]+)]")
        .multi_line(true)
        .build()
        .unwrap();

    let mut tags: HashMap<&str, String> = HashMap::new();
    for (_full, [m]) in re.captures_iter(pgn).map(|c| c.extract()) {
        let tag: Vec<&str> = m.split(' ').collect();
        let v = tag[1].replace("\"", "");
        tags.insert(tag[0], v);
    }

    tags
}

fn merge_results(a: GameResult, b: GameResult) -> GameResult {
    let mut merged: GameResult = b.clone();

    for (a_key, a_value) in a.clone().into_iter() {
        let value_in_b = b.get(&a_key);
        merged.insert(a_key, match value_in_b {
            Some(v) => v + a_value,
            None => a_value
        });
    }

    merged
}

fn merge_entries(a: &Entry, b: &Entry) -> Entry {
    Entry {
        count: a.count + b.count,
        average_accuracy: match (a.average_accuracy, b.average_accuracy) {
            (Some(a_avg), Some(b_avg)) => Some((a_avg + b_avg) / 2.0),
            (Some(a_avg), None) => Some(a_avg),
            (None, Some(b_avg)) => Some(b_avg),
            (None, None) => None
        },
        results: merge_results(a.clone().results, b.clone().results),
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
    let black = tags.get("Black");
    let white = tags.get("White");

    match black {
        Some(username) if *username == cli.username => return Some("Black"),
        Some(_) => (),
        None => (),
    };

    match white {
        Some(username) if *username == cli.username => Some("White"),
        Some(_) => None,
        None => None,
    }
}

fn get_opening_name<'tags>(tags: &'tags ParsedPgnTags) -> Option<&'tags str> {
    match tags.get("ECOUrl") {
        Some(eco_url) => {
            let parts: Vec<&str> = eco_url.split('/').collect();
            Some(parts[parts.len() - 1])
        },
        None => None
    }
}

fn get_game_result(user_pieces: String, game: &Game) -> HashMap<String, u32> {
    let result: String = if user_pieces == "Black" {
        game.black.result.to_owned()
    } else {
        game.white.result.to_owned()
    };

    let mut results: HashMap<String, u32> = HashMap::new();
    results.insert(result, 1);
    results
}

fn add_game_to_output(cli: &Cli, json: &mut OutputJson, game: &Game) {
    let tags: ParsedPgnTags = parse_pgn_tags(&game.pgn);

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

    let results: HashMap<String, u32> = get_game_result(user_pieces.to_string(), game);

    let new_entry = Entry {
        average_accuracy: accuracy,
        count: 1,
        results,
    };

    let inserted_entry: Entry = match json.get(opening_name) {
        Some(existing_entry) => merge_entries(existing_entry, &new_entry),
        None => new_entry
    };

    json.insert(opening_name.to_string(), inserted_entry);
}

async fn run(cli: &Cli) {
    let now = Local::now();
    let month = now.date_naive().month();

    let client = reqwest::Client::new();
    let response_futures = (1..month).map(|m| get_month_games(cli, &client, m));
    let responses = join_all(response_futures).await;

    let mut json: OutputJson = HashMap::new();
    for response in responses {
        match response {
            None => continue,
            Some(r) => {
                for game in r.games {
                    add_game_to_output(cli, &mut json, &game);
                }
            },
        }
    }

    let output = Output {
        json
    };

    let output_string = serde_json::to_string(&output.json);
    match output_string {
        Ok(str) => println!("{}", str),
        Err(e) => println!("{}", e),
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    run(&cli).await;
}
