use csv::ReaderBuilder;
use discorec::{Dataset, RecommenderBuilder};
use pgvector::Vector;
use postgres::{Client, NoTls};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() {
    // https://grouplens.org/datasets/movielens/100k/
    let movielens_path = std::env::var("MOVIELENS_100K_PATH").expect("Set MOVIELENS_100K_PATH");

    let mut client = Client::configure()
        .host("localhost")
        .dbname("pgvector_rust_test")
        .user(std::env::var("USER").unwrap().as_str())
        .connect(NoTls)
        .unwrap();

    client.execute("CREATE EXTENSION IF NOT EXISTS vector", &[]).unwrap();
    client.execute("DROP TABLE IF EXISTS users", &[]).unwrap();
    client.execute("DROP TABLE IF EXISTS movies", &[]).unwrap();
    client.execute("CREATE TABLE users (id integer PRIMARY KEY, factors vector(20))", &[]).unwrap();
    client.execute("CREATE TABLE movies (name text PRIMARY KEY, factors vector(20))", &[]).unwrap();

    let data = load_movielens(Path::new(&movielens_path));
    let recommender = RecommenderBuilder::new().factors(20).fit_explicit(&data);

    for user_id in recommender.user_ids() {
        let factors = Vector::from(recommender.user_factors(user_id).unwrap().to_vec());
        client.execute("INSERT INTO users (id, factors) VALUES ($1, $2)", &[&user_id, &factors]).unwrap();
    }

    for item_id in recommender.item_ids() {
        let factors = Vector::from(recommender.item_factors(item_id).unwrap().to_vec());
        client.execute("INSERT INTO movies (name, factors) VALUES ($1, $2)", &[&item_id, &factors]).unwrap();
    }

    let movie = "Star Wars (1977)";
    println!("Item-based recommendations for {}", movie);
    for row in client.query("SELECT name FROM movies WHERE name != $1 ORDER BY factors <=> (SELECT factors FROM movies WHERE name = $1) LIMIT 5", &[&movie]).unwrap() {
        let name: &str = row.get(0);
        println!("- {}", name);
    }

    let user_id = 123;
    println!("\nUser-based recommendations for user {}", user_id);
    for row in client.query("SELECT name FROM movies ORDER BY factors <#> (SELECT factors FROM users WHERE id = $1) LIMIT 5", &[&user_id]).unwrap() {
        let name: &str = row.get(0);
        println!("- {}", name);
    }
}

fn load_movielens(path: &Path) -> Dataset<i32, String> {
    // read movies, removing invalid UTF-8 bytes
    let mut movies = HashMap::new();
    let mut movies_file = File::open(path.join("u.item")).unwrap();
    let mut buf = Vec::new();
    movies_file.read_to_end(&mut buf).unwrap();
    let movies_data = String::from_utf8_lossy(&buf);
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'|')
        .from_reader(movies_data.as_bytes());
    for record in rdr.records() {
        let row = record.unwrap();
        movies.insert(row[0].to_string(), row[1].to_string());
    }

    // read ratings and create dataset
    let mut data = Dataset::new();
    let ratings_file = File::open(path.join("u.data")).unwrap();
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(ratings_file);
    for record in rdr.records() {
        let row = record.unwrap();
        data.push(
            row[0].parse::<i32>().unwrap(),
            movies.get(&row[1].to_string()).unwrap().to_string(),
            row[2].parse().unwrap(),
        );
    }

    data
}
