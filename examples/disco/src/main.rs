use discorec::{Dataset, RecommenderBuilder};
use pgvector::Vector;
use postgres::{Client, NoTls};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    // https://grouplens.org/datasets/movielens/100k/
    let movielens_path = std::env::var("MOVIELENS_100K_PATH").or(Err("Set MOVIELENS_100K_PATH"))?;

    let mut client = Client::configure()
        .host("localhost")
        .dbname("pgvector_example")
        .user(std::env::var("USER")?.as_str())
        .connect(NoTls)?;

    client.execute("CREATE EXTENSION IF NOT EXISTS vector", &[])?;
    client.execute("DROP TABLE IF EXISTS users", &[])?;
    client.execute("DROP TABLE IF EXISTS movies", &[])?;
    client.execute(
        "CREATE TABLE users (id integer PRIMARY KEY, factors vector(20))",
        &[],
    )?;
    client.execute(
        "CREATE TABLE movies (name text PRIMARY KEY, factors vector(20))",
        &[],
    )?;

    let data = load_movielens(Path::new(&movielens_path))?;
    let recommender = RecommenderBuilder::new().factors(20).fit_explicit(&data);

    for user_id in recommender.user_ids() {
        let factors = Vector::from(recommender.user_factors(user_id).unwrap().to_vec());
        client.execute(
            "INSERT INTO users (id, factors) VALUES ($1, $2)",
            &[&user_id, &factors],
        )?;
    }

    for item_id in recommender.item_ids() {
        let factors = Vector::from(recommender.item_factors(item_id).unwrap().to_vec());
        client.execute(
            "INSERT INTO movies (name, factors) VALUES ($1, $2)",
            &[&item_id, &factors],
        )?;
    }

    let movie = "Star Wars (1977)";
    println!("Item-based recommendations for {}", movie);
    for row in client.query("SELECT name FROM movies WHERE name != $1 ORDER BY factors <=> (SELECT factors FROM movies WHERE name = $1) LIMIT 5", &[&movie])? {
        let name: &str = row.get(0);
        println!("- {}", name);
    }

    let user_id = 123;
    println!("\nUser-based recommendations for user {}", user_id);
    for row in client.query("SELECT name FROM movies ORDER BY factors <#> (SELECT factors FROM users WHERE id = $1) LIMIT 5", &[&user_id])? {
        let name: &str = row.get(0);
        println!("- {}", name);
    }

    Ok(())
}

fn load_movielens(path: &Path) -> Result<Dataset<i32, String>, Box<dyn Error>> {
    // read movies, removing invalid UTF-8 bytes
    let mut movies = HashMap::new();
    let mut movies_file = File::open(path.join("u.item"))?;
    let mut buf = Vec::new();
    movies_file.read_to_end(&mut buf)?;
    let movies_data = String::from_utf8_lossy(&buf);
    let rdr = BufReader::new(movies_data.as_bytes());
    for line in rdr.lines() {
        let line = line?;
        let row: Vec<_> = line.split('|').collect();
        movies.insert(row[0].to_string(), row[1].to_string());
    }

    // read ratings and create dataset
    let mut data = Dataset::new();
    let ratings_file = File::open(path.join("u.data"))?;
    let rdr = BufReader::new(ratings_file);
    for line in rdr.lines() {
        let line = line?;
        let row: Vec<_> = line.split('\t').collect();
        data.push(
            row[0].parse::<i32>()?,
            movies.get(row[1]).unwrap().to_string(),
            row[2].parse()?,
        );
    }

    Ok(data)
}
