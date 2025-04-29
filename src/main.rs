use chrono::Datelike;
use chrono::Local;
use chrono::SubsecRound;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::collections::HashMap;
use std::env;
use std::fmt::Debug;
use std::option::Option;
use std::result::Result;
use std::vec::Vec;

struct Cli {
    pieces: String,
    time_class: String,
    username: String,
    year: String,
}

#[derive(Clone, Debug, Serialize)]
struct Entry {
    count: u32,
    results: HashMap<String, u32>,
    average_accuracy: Option<f32>,
}

#[derive(Deserialize)]
struct Accuracies {
    black: f32,
    white: f32,
}

#[derive(Deserialize)]
struct PieceInfo {
    result: String,
    username: String,
}

#[derive(Deserialize)]
struct Game {
    accuracies: Option<Accuracies>,
    pgn: String,
    black: PieceInfo,
    white: PieceInfo,
}

#[derive(Deserialize)]
struct ResponseJson {
    games: Vec<Game>
}

#[derive(Serialize)]
struct Output {
    json: HashMap<String, Entry>
}

async fn get_month_games(cli: &Cli, month: u32) -> Option<ResponseJson> {
    let result = reqwest::Client::new()
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

type ParsedPgnTags<'a> = HashMap<&'a str, String>;

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

fn merge_results(a: HashMap<String, u32>, b: HashMap<String, u32>) -> HashMap<String, u32> {
    let mut merged: HashMap<String, u32> = HashMap::new();

    for (a_key, a_value) in a.clone().into_iter() {
        let value_in_b = b.get(&a_key);
        merged.insert(a_key, match value_in_b {
            Some(v) => v + a_value,
            None => a_value
        });
    }

    merged
}

fn merge_entries(a: Entry, b: Entry) -> Entry {
    { Entry {
        count: a.count + b.count,
        average_accuracy: match (a.average_accuracy, b.average_accuracy) {
            (Some(a_avg), Some(b_avg)) => Some((a_avg + b_avg) / 2.0),
            (Some(a_avg), None) => Some(a_avg),
            (None, Some(b_avg)) => Some(b_avg),
            (None, None) => None
        },
        results: merge_results(a.results, b.results),
    }}
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

#[tokio::main]
async fn main() {
    let pieces = env::args().nth(1).expect("The 'pieces' argument is required.");
    let time_class = env::args().nth(2).expect("The 'time_class' argument is required.");
    let username = env::args().nth(3).expect("The 'username' argument is required.");
    let year = env::args().nth(4).expect("The 'year' argument is required.");

    let cli = Cli {
        pieces,
        time_class,
        username,
        year
    };

    let now = Local::now();
    let month = now.date_naive().month();

    let response_futures = (1..month).map(|m| get_month_games(&cli, m));
    let responses = join_all(response_futures).await;

    let mut json: HashMap<String, Entry> = HashMap::new();
    for response in responses {
        match response {
            None => continue,
            Some(r) => {
                for game in r.games {
                    let tags: ParsedPgnTags = parse_pgn_tags(&game.pgn);

                    let user_pieces = match get_user_pieces(&cli, &tags) {
                        Some(user_pieces) => user_pieces,
                        None => continue,
                    };

                    let accuracy: Option<f32> = get_game_accuracy(user_pieces, &game);

                    let opening_name = match get_opening_name(&tags) {
                        Some(opening_name) => opening_name,
                        None => continue,
                    };

                    let result: String = if user_pieces == "Black" {
                        game.black.result
                    } else {
                        game.white.result
                    };

                    let mut results: HashMap<String, u32> = HashMap::new();
                    results.insert(result, 1);

                    let new_entry: Entry = { Entry {
                        average_accuracy: accuracy,
                        count: 1,
                        results,
                    }};

                    let inserted_entry: Entry = match json.get(opening_name) {
                        Some(existing_entry) => merge_entries(existing_entry.clone(), new_entry),
                        None => new_entry
                    };

                    json.insert(opening_name.to_string(), inserted_entry);
                }

            },
        }
    }

    let output = { Output {
        json: json.clone()
    }};

    let output_string = serde_json::to_string(&output.json);
    match output_string {
        Ok(str) => println!("{}", str),
        Err(e) => println!("{}", e),
    }
}
