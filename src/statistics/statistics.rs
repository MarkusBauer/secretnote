use std::env;
use std::fs;
use std::str;
use redis_async::{client, resp_array};
use redis_async::resp::RespValue;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};


fn to_int(rv: RespValue) -> i64 {
    match rv {
        RespValue::Integer(x) => x,
        RespValue::BulkString(x) => str::from_utf8(&x).unwrap().parse().unwrap_or(0),
        _ => 0
    }
}


#[derive(Serialize, Deserialize, Default)]
struct StatisticsEntry {
    epoch: String,

    note_store_count_total: i64,
    note_store_bytes_total: i64,
    note_retrieve_count_total: i64,
    chat_message_count_total: i64,
    chat_message_bytes_total: i64,
    note_store_count: i64,
    note_store_bytes: i64,
    note_retrieve_count: i64,
    chat_message_count: i64,
    chat_message_bytes: i64,
}


#[tokio::main]
async fn main() {
    let matches = clap::App::new("SecretNote Statistics")
        //.version("1.0")
        //.author("...")
        .about("SecretNote statistics exporter")
        .arg(clap::Arg::new("redis")
            .long("redis")
            .value_name("HOST:PORT")
            .about("Sets which ip/port should be bound")
            .takes_value(true))
        .arg(clap::Arg::new("redis-db")
            .long("redis-db")
            .value_name("DB")
            .about("database number")
            .takes_value(true))
        .arg(clap::Arg::new("redis-auth")
            .long("redis-auth")
            .value_name("PASSWORD")
            .about("Sets a password for the redis database")
            .takes_value(true))
        .get_matches();
    let redis: String = matches.value_of("redis").unwrap_or(&env::var("SECRETNOTE_REDIS").unwrap_or("127.0.0.1:6379".into())).into();
    let redis_db: u32 = matches.value_of_t("redis-db").unwrap_or(env::var("SECRETNOTE_REDIS_DB").unwrap_or("0".into()).parse().expect("Redis database must be a number"));
    let redis_auth = if let Some(x) = matches.value_of("redis-auth") { Some(String::from(x)) } else { env::var("SECRETNOTE_REDIS_AUTH").ok() };

    // Prepare redis connection
    let connection = client::paired_connect(&redis.parse().unwrap()).await.expect("Connection to redis failed");
    if let Some(redis_auth_str) = redis_auth {
        connection.send_and_forget(resp_array!["AUTH", redis_auth_str]);
    }
    connection.send_and_forget(resp_array!["SELECT", format!("{}", redis_db)]);

    // Get statistics
    let epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
    let mut entry = StatisticsEntry { epoch: format!("{}", epoch.as_millis()), ..Default::default() };
    entry.note_store_count_total = to_int(connection.send::<RespValue>(resp_array!["GET", "secretnote-stats:note-store-count"]).await.expect("note-store-count"));
    entry.note_store_bytes_total = to_int(connection.send::<RespValue>(resp_array!["GET", "secretnote-stats:note-store-bytes"]).await.expect("note-store-bytes"));
    entry.note_retrieve_count_total = to_int(connection.send::<RespValue>(resp_array!["GET", "secretnote-stats:note-retrieve-count"]).await.expect("note-retrieve-count"));
    entry.chat_message_count_total = to_int(connection.send::<RespValue>(resp_array!["GET", "secretnote-stats:chat-message-count"]).await.expect("chat-message-count"));
    entry.chat_message_bytes_total = to_int(connection.send::<RespValue>(resp_array!["GET", "secretnote-stats:chat-message-bytes"]).await.expect("chat-message-bytes"));

    // Read last statistics
    entry.note_store_count = entry.note_store_count_total;
    entry.note_store_bytes = entry.note_store_bytes_total;
    entry.note_retrieve_count = entry.note_retrieve_count_total;
    entry.chat_message_count = entry.chat_message_count_total;
    entry.chat_message_bytes = entry.chat_message_bytes_total;

    if let Ok(last) = fs::read_to_string("statistics_current.json") {
        if let Ok(last_stats) = serde_json::from_str::<StatisticsEntry>(last.as_str()) {
            entry.note_store_count -= last_stats.note_store_count_total;
            entry.note_store_bytes -= last_stats.note_store_bytes_total;
            entry.note_retrieve_count -= last_stats.note_retrieve_count_total;
            entry.chat_message_count -= last_stats.chat_message_count_total;
            entry.chat_message_bytes -= last_stats.chat_message_bytes_total;
        } else {
            eprintln!("Could not parse statistics_current.json");
        }
    } else {
        eprintln!("File statistics_current.json not found");
    }

    // Write out
    // let json = serde_json::to_string(&entry).expect("json failed");
    let json = serde_json::to_string_pretty(&entry).expect("json failed");
    println!("{}", json);
    fs::write("statistics_current.json", json).expect("Could not save json file!");
}