use std::str;
use std::io::{self, Read};
use aes_gcm::Aes128Gcm;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use aes_gcm::aead::{Aead, NewAead, generic_array::GenericArray};
use regex::Regex;
use lazy_static::{lazy_static};


#[allow(dead_code)]
fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|x| format!("{:02x}", x)).collect::<String>()
}


#[derive(Serialize, Deserialize)]
pub struct Note { text: String }

#[derive(Serialize)]
pub struct NoteRequest { data: String }

#[derive(Deserialize)]
pub struct NoteResponse { ident: String, admin_ident: String }

#[derive(Serialize)]
struct RetrieveNoteRequest { ident: String }

#[derive(Deserialize)]
struct RetrieveNoteResponse { #[allow(dead_code)] ident: String, data: String }

struct NoteLinks {
    public_link: String,
    admin_link: String,
}


async fn secretnote_note_store(host: &str, text: &str) -> NoteLinks {
    let data = serde_json::to_vec(&Note { text: text.into() }).expect("JSON serialize failed");
    // encrypt
    let mut gen = OsRng::default();
    let mut key = [0u8; 16];
    let mut iv = [0u8; 12];
    gen.fill_bytes(&mut key);
    gen.fill_bytes(&mut iv);
    let cipher = Aes128Gcm::new(&GenericArray::from(key));
    let mut c = cipher.encrypt(&GenericArray::from(iv), data.as_slice()).expect("encryption failed");
    let mut ciphertext = Vec::from(iv);
    ciphertext.append(&mut c);
    // submit
    let req = NoteRequest { data: base64::encode(ciphertext) };
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
    return NoteLinks {
        public_link: format!("{}/note/{}#{}", host.trim_end_matches("/"), body.ident, base64::encode_config(key, base64::URL_SAFE_NO_PAD)),
        admin_link: format!("{}/note/admin/{}#{}", host.trim_end_matches("/"), body.admin_ident, base64::encode_config(key, base64::URL_SAFE_NO_PAD)),
    };
}


struct ParsedNoteUrl { host: String, ident: String, key: String }

fn secretnote_parse_note_url(url: &str) -> Option<ParsedNoteUrl> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^(https?://[^/]+)/note/([A-Za-z0-9_-]{28})#([A-Za-z0-9_-]+)$").unwrap();
    }
    if let Some(cap) = RE.captures(url) {
        Some(ParsedNoteUrl{host: cap[1].into(), ident: cap[2].into(), key: cap[3].into()})
    } else {
        None
    }
}

async fn secretnote_note_retrieve(link: &ParsedNoteUrl) -> String {
    // Parse key
    let key = base64::decode_config(link.key.as_str(), base64::URL_SAFE_NO_PAD).expect("Invalid key given");
    let cipher = Aes128Gcm::new(&GenericArray::from_slice(key.as_slice()));
    // Retrieve link
    let req = RetrieveNoteRequest { ident: (&link.ident).into() };
    let mut url: String = link.host.trim_end_matches("/").into();
    url.push_str("/api/note/retrieve");
    let client = reqwest::Client::new();
    let response = client.post(&url).json(&req).send().await.expect("Could not reach server");
    if response.status() != 200 {
        panic!("Server response {}", response.status());
    }
    let body: RetrieveNoteResponse = response.json().await.expect("Invalid JSON");
    // Decrypt
    let c_and_iv = base64::decode(body.data.as_str()).unwrap();
    let p = cipher.decrypt(&GenericArray::from_slice(&c_and_iv.as_slice()[..12]), &c_and_iv.as_slice()[12..]).expect("Decryption failed");
    let data: Note = serde_json::from_slice(p.as_slice()).expect("Invalid JSON after decryption");
    return data.text;
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
        .arg(clap::Arg::new("URL")
            .about("A note URL to retrieve"))
        .get_matches();
    let host = matches.value_of("host").unwrap_or("https://secretnote.mk-bauer.de");
    let url = matches.value_of("URL");

    if let Some(url) = url {
        // Retrieve note from url
        let parsed_url = secretnote_parse_note_url(url).expect("Invalid URL!");
        let text = secretnote_note_retrieve(&parsed_url).await;
        println!("{}", text);

    } else {
        // Store stdin as note
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).expect("Could not read from stdin");
        let links = secretnote_note_store(host, &buffer).await;
        println!("{}", &links.public_link);
        println!("{}", &links.admin_link);
    }
}