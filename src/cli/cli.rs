use std::str;
use std::io::{self, Read};
use aes_gcm::Aes128Gcm;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use aes_gcm::aead::{Aead, NewAead, generic_array::GenericArray};



fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|x| format!("{:02x}", x)).collect::<String>()
}

#[derive(Serialize)]
pub struct Note {
    /// base64-encoded crypted data
    data: String
}

#[derive(Deserialize)]
pub struct NoteResponse { ident: String, admin_ident: String }

struct NoteLinks {
    public_link: String,
    admin_link: String
}


async fn secretnote_note_store(host: &str, text: &Vec<u8>) -> NoteLinks {
    // encrypt
    let mut gen = OsRng::default();
    let mut key = [0u8; 16];
    let mut iv = [0u8; 12];
    gen.fill_bytes(&mut key);
    gen.fill_bytes(&mut iv);
    let cipher = Aes128Gcm::new(&GenericArray::from(key));
    let mut c = cipher.encrypt(&GenericArray::from(iv), text.as_slice()).expect("encryption failed");
    let mut ciphertext = Vec::from(iv);
    ciphertext.push(0);
    ciphertext.push(0);
    ciphertext.push(0);
    ciphertext.push(0);
    ciphertext.append(&mut c);
    println!("key = {}", to_hex(key.as_ref()));
    println!("iv  = {}", to_hex(iv.as_ref()));
    println!("c   = {}", to_hex(ciphertext.as_ref()));
    // submit
    let req = Note{data: base64::encode(ciphertext)};
    let mut url: String = host.trim_end_matches("/").into();
    url.push_str("/api/note/store");
    let client = reqwest::Client::new();
    let response = client.post(&url).json(&req).send().await.expect("Could not reach server");
    //let mut response = Client::default().post(&url).content_type("application/json; charset=utf-8").send_json(&req).await.expect("Could not reach server");
    if response.status() != 200 {
        panic!("Server response {}", response.status());
    }
    let body: NoteResponse = response.json().await.expect("Invalid JSON");
    // generate URLs
    return NoteLinks{
        public_link: format!("{}/note/{}#{}", host.trim_end_matches("/"), body.ident, base64::encode_config(key, base64::URL_SAFE_NO_PAD)),
        admin_link: format!("{}/note/admin/{}#{}", host.trim_end_matches("/"), body.admin_ident, base64::encode_config(key, base64::URL_SAFE_NO_PAD)),
    }
}


#[tokio::main]
async fn main() {
    let matches = clap::App::new("SecretNote CLI")
        .about("SecretNote command line")
        .arg(clap::Arg::new("host")
            .long("host")
            .value_name("HOST")
            .about("Set the SecretNote host to use")
            .takes_value(true))
        .get_matches();
    let host = matches.value_of("host").unwrap_or("https://secretnote.mk-bauer.de");


    // Store stdin as note
    let mut buffer: Vec<u8> = vec!();
    io::stdin().read_to_end(&mut buffer).expect("Could not read from stdin");
    let links = secretnote_note_store(host, &buffer).await;
    println!("{}", &links.public_link);
    println!("{}", &links.admin_link);
}