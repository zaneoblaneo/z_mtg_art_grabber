pub mod error;
use error::Error;

use serde_json::Value;

use std::fs::{ DirBuilder, File };
use std::io::Write;

fn print_usage(args: &Vec<String>) {
    println!("Valid usage: {} [path_to_deck.json]", args[0]);
}

#[derive(Debug, Clone)]
struct Card {
    name: String,
    front_url: Option<String>,
    back_url: Option<String>,
}

fn populate_cards(json: Value) -> Result<(String, Vec<Card>), Error> {
    let deck_name: String = json["name"].as_str().ok_or(Error::InvalidJsonKey("name".to_owned(), json.clone()))?.to_owned();
    let sections: Value = json["sections"].clone();
    let primary: Value = sections["primary"].clone();
    let secondary: Value = sections["secondary"].clone();
    let mut section_vec: Vec<Value> = Vec::new();
    for v in primary.as_array().ok_or(Error::InvalidJsonType("Array".to_owned(), primary.clone()))? {
        section_vec.push(v.clone());
    }
    for v in secondary.as_array().ok_or(Error::InvalidJsonType("Array".to_owned(), secondary.clone()))? {
        section_vec.push(v.clone());
    }

    let mut cards: Vec<Card> = Vec::new();
    for section_name in section_vec {
        let entries = &json["entries"];
        let section = &entries[section_name.as_str().ok_or(Error::InvalidJsonType("String".to_owned(), entries.clone()))?];
        for card in section.as_array().ok_or(Error::InvalidJsonType("Array".to_owned(), section.clone()))? {
            for _ in 0..card["count"].as_u64().ok_or(Error::InvalidJsonType("u64".to_owned(), card.clone()))? {
                let card_digest: Value = card["card_digest"].clone();
                if card_digest.is_null() {
                    continue;
                }
                let card_name: String = card_digest["name"].as_str().ok_or(Error::InvalidJsonType("String".to_owned(), card_digest.clone()))?.to_owned();
                let image_uris = card_digest["image_uris"].clone();
                let front: Option<String> = if image_uris["front"].is_string() {
                    Some(image_uris["front"].as_str().ok_or(Error::InvalidJsonType("String".to_owned(), image_uris["front"].clone()))?.to_owned().replace("large", "png").replace(".jpg", ".png").split("?").collect::<Vec<&str>>()[0].to_owned())
                } else {
                    None
                };
                let back: Option<String> = if image_uris["back"].is_string() {
                    Some(image_uris["back"].as_str().ok_or(Error::InvalidJsonType("String".to_owned(), image_uris["back"].clone()))?.to_owned().replace("large", "png").replace(".jpg", ".png").split("?").collect::<Vec<&str>>()[0].to_owned())
                } else {
                    None
                };
                cards.push(Card { name: card_name, front_url: front, back_url: back });
            }
        }
    }
    Ok((deck_name, cards))
}

const EMPTY_STR: &'static str = "";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_usage(&args);
        std::process::exit(1);
    }

    let json_file_path = args[1].clone();
    let json_file_data = std::fs::read_to_string(json_file_path).unwrap();
    let json: Value = serde_json::from_str(json_file_data.as_str())?;
    let dir_builder = DirBuilder::new();
    let (deck_name, cards): (String, Vec<Card>) = populate_cards(json)?;
    println!("Found: {} cards in `{}`", cards.len(), deck_name.as_str());
    dir_builder.create(deck_name.as_str()).unwrap();
    for (i, card) in cards.iter().enumerate() {
        println!("Downloading card {} of {}", i, cards.len() - 1);
        let card_name_replaced = card.name.as_str().replace("/", "(slash)");
        let out_path = format!("{}/{i} - {}", deck_name.as_str(), card_name_replaced);
        dir_builder.create(&out_path).unwrap();
        let client = reqwest::Client::new();
        if let Some(front) = &card.front_url {
            let response = client.get(front.as_str()).send().await?;
            let headers = response.headers().clone();
            let content_disposition = headers.get("content-disposition").ok_or(Error::MissingHeader("content-disposition".to_owned(), format!("{:#?}", headers.clone())))?;
            let tmp: Vec<&str> = if let Ok(t) = content_disposition.to_str() {
                t.split("filename=").collect()
            } else {
                let mut out: Vec<&str> = Vec::new();
                out.push(EMPTY_STR);
                out.push("front.jpg");
                out
            };
            let filename = tmp.get(1).ok_or(Error::InvalidHeader("content-disposition".to_owned(), format!("{:#?}", headers.clone())))?;
            let filename = filename.replace("\"", "").replace("*", "(asterisk)");
            let out_file_path = format!("{}/{}", out_path, filename);
            let mut file = File::create(out_file_path).unwrap();
            let b = response.bytes().await?;
            file.write_all(&b).unwrap();
        }
        if let Some(back) = &card.back_url {
            let response = client.get(back.as_str()).send().await?;
            let headers = response.headers().clone();
            let content_disposition = headers.get("content-disposition").ok_or(Error::MissingHeader("content-disposition".to_owned(), format!("{:#?}", headers.clone())))?;
            let tmp: Vec<&str> = if let Ok(t) = content_disposition.to_str() {
                t.split("filename=").collect()
            } else {
                let mut out: Vec<&str> = Vec::new();
                out.push(EMPTY_STR);
                out.push("back.jpg");
                out
            };
            let filename = tmp.get(1).ok_or(Error::InvalidHeader("content-disposition".to_owned(), format!("{:#?}", headers.clone())))?;
            let filename = filename.replace("\"", "").replace("*", "(asterisk)");
            let out_file_path = format!("{}/{}", out_path, filename);
            let mut file = File::create(out_file_path).unwrap();
            let b = response.bytes().await?;
            file.write_all(&b).unwrap();
        }
    }
    
    Ok(())
}
