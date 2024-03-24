#![allow(dead_code)]
use indicatif::{ProgressBar, ProgressStyle};
mod models;
use anyhow::{Ok, Result};
use models::{Album, Asset};
use reqwest::{header::HeaderMap, Client, Response};
use serde_json::json;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufReader, LineWriter, Write},
    path::PathBuf,
};

use crate::models::{AlbumSaveFile, FavouriteCollection, FavouriteCollectionSaveFile};

use clap::{Parser, Subcommand};

struct ServerInfo {
    client: Client,
    server: String,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path of the savefile
    #[arg(short, long, value_name = "FILE")]
    file_name: PathBuf,

    /// Name of the person to greet
    #[arg(short, long)]
    server: String,

    /// REST API Key
    #[arg(short, long)]
    key: String,
}

#[derive(Subcommand)]
#[command(version, about, long_about = None)]
enum Commands {
    /// Save Immich Albums and Favorite assets
    Save,
    /// Load Immich Albums and Favorite assets
    Load,
}

async fn get_client(key: &str) -> Result<Client, reqwest::Error> {
    let mut header = HeaderMap::new();
    header.insert("Accept", "application/json".parse().unwrap());
    header.insert("x-api-key", key.parse().unwrap());
    let client = reqwest::Client::builder().default_headers(header);
    client.build()
}

async fn call_api(server_info: &ServerInfo, endpoint: &str) -> Result<Response, reqwest::Error> {
    server_info
        .client
        .get(format!("{}{endpoint}", &server_info.server))
        .send()
        .await
}

async fn get_all_albums(server_info: &ServerInfo) -> Result<Vec<Album>, reqwest::Error> {
    let res = call_api(server_info, "/api/album").await?;
    res.json().await
}

async fn get_all_assets(server_info: &ServerInfo) -> Result<Vec<Asset>, reqwest::Error> {
    println!(
        "{}",
        console::style("Loading all assets, this may take a while.").bold()
    );
    let res = call_api(server_info, "/api/asset").await?;
    println!(
        "{}",
        console::style("✅ All assets have been loaded")
            .bold()
            .color256(40)
    );
    res.json().await
}
async fn get_album_content(
    server_info: &ServerInfo,
    album: Album,
) -> Result<Album, reqwest::Error> {
    let res = call_api(
        server_info,
        format!("/api/album/{}", album.id.unwrap()).as_str(),
    )
    .await?;
    res.json().await
}

async fn get_all_favorites(server_info: &ServerInfo) -> Result<Vec<Asset>, reqwest::Error> {
    let res = call_api(server_info, "/api/asset?isFavorite=true").await?;
    res.json().await
}

async fn load_all_albums(
    server_info: &ServerInfo,
    spinner_style: ProgressStyle,
) -> Result<Vec<Album>, anyhow::Error> {
    println!("{}", console::style("Loading all Albums").bold().bright());
    let mut contents: Vec<Album> = Vec::new();
    let albums = get_all_albums(server_info).await?;
    let pb = ProgressBar::new(albums.len() as u64).with_style(spinner_style);
    pb.set_prefix("Loading: ");
    for album in albums {
        pb.set_message(album.album_name.clone());
        contents.push(get_album_content(server_info, album).await?);
        pb.inc(1);
    }
    pb.finish_and_clear();

    Ok(contents)
}
async fn save_albums(
    server_info: &ServerInfo,
    save_file: &File,
    spinner_style: ProgressStyle,
) -> Result<(), anyhow::Error> {
    let mut file_buff = LineWriter::new(save_file);
    let albums: Vec<Album> = load_all_albums(server_info, spinner_style).await?;
    let albums: Vec<AlbumSaveFile> = albums.into_iter().map(AlbumSaveFile::from_album).collect();
    let out = serde_yaml::to_string(&albums)?;
    println!("{}", console::style("Saving Ablums").bold());
    file_buff.write_all(out.as_bytes())?;
    Ok(())
}

async fn save_favs(server_info: &ServerInfo, save_file: &File) -> Result<(), anyhow::Error> {
    let mut file_buff = LineWriter::new(save_file);
    println!(
        "{}",
        console::style("Loading all Favourite assets")
            .bold()
            .bright()
    );

    let favs = FavouriteCollection {
        favorites: get_all_favorites(server_info).await?,
    };
    let favs = FavouriteCollectionSaveFile::from_fav_collection(favs);
    let out = serde_yaml::to_string(&vec![favs])?;
    file_buff.write_all(out.as_bytes())?;
    println!("{}", console::style("Saving Favorites").bold());
    Ok(())
}

async fn read_savefile(file: &File) -> Result<(Vec<Album>, FavouriteCollection), anyhow::Error> {
    let data: serde_yaml::Value = serde_yaml::from_reader(BufReader::new(file)).unwrap();
    let mut favorites: FavouriteCollectionSaveFile = FavouriteCollectionSaveFile::new();
    let mut albumns: Vec<AlbumSaveFile> = Vec::new();
    for item in data.as_sequence().unwrap() {
        if let serde_yaml::Value::Mapping(mapping) = item {
            if mapping.contains_key("favorites") {
                favorites = serde_yaml::from_value(item.clone()).unwrap()
            } else {
                albumns.push(serde_yaml::from_value(item.clone()).unwrap())
            };
        }
    }

    let albumns: Vec<Album> = albumns.into_iter().map(|x| x.to_album()).collect();
    let favorites: FavouriteCollection = favorites.to_fav_collection();
    Ok((albumns, favorites))
}

async fn fav_assets(server_info: &ServerInfo, assets_id: Vec<String>) -> Result<(), anyhow::Error> {
    let body = json!({"ids":assets_id,
"isFavorite":true});
    server_info
        .client
        .put(format!("{}/api/asset", server_info.server))
        .json(&body)
        .send()
        .await?;
    Ok(())
}

async fn modify_favs(
    server_info: &ServerInfo,
    favs: FavouriteCollection,
) -> Result<(), anyhow::Error> {
    if favs.favorites.is_empty() {
        return Ok(());
    }

    let all_assets: Vec<Asset> = get_all_assets(server_info).await?;

    // Create a HashMap to store asset IDs for faster lookups
    let mut asset_map: HashMap<&str, Vec<String>> = HashMap::new();
    for asset in &all_assets {
        if let Some(name) = &asset.original_file_name {
            asset_map
                .entry(name)
                .or_default()
                .push(asset.id.clone().unwrap());
        }
    }

    let mut ids_to_fav: Vec<String> = Vec::new();
    for fav in favs.favorites {
        if let Some(name) = fav.original_file_name {
            if let Some(ids) = asset_map.get(name.as_str()) {
                ids_to_fav.extend(ids.clone());
            }
        }
    }

    println!(
        "{}",
        console::style(format!("Setting {} favs ❤️", ids_to_fav.len()))
    );

    fav_assets(server_info, ids_to_fav).await?;
    Ok(())
}

async fn modify_albums(
    server_info: &ServerInfo,
    albums: Vec<Album>,
    spinner_style: ProgressStyle,
) -> Result<(), anyhow::Error> {
    if albums.is_empty() {
        return Ok(());
    };

    let all_albums = load_all_albums(server_info, spinner_style).await?;
    let all_albums_map: HashMap<String, Album> =
        all_albums
            .into_iter()
            .fold(HashMap::new(), |mut acc, album| {
                acc.insert(album.album_name.clone(), album);
                acc
            });
    for save_file_album in albums.iter() {
        if all_albums_map.contains_key(&save_file_album.album_name) {
            // Albums exists and needs to be modified
            todo!();
        } else {
            // Make new album
            todo!();
        }
    }

    Ok(())
}

async fn modify_results(
    server_info: &ServerInfo,
    results: (Vec<Album>, FavouriteCollection),
) -> Result<(), anyhow::Error> {
    modify_favs(server_info, results.1).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    let cli = Cli::parse();
    let save_file = cli.file_name;
    let client = get_client(&cli.key).await?;
    let server_info = ServerInfo {
        client,
        server: cli.server,
    };

    match cli.command {
        Commands::Save => {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(save_file)
                .unwrap();

            println!("{}", console::style("Saving into savefile").bold());
            save_albums(&server_info, &file, spinner_style).await?;
            save_favs(&server_info, &file).await?;
        }
        Commands::Load => {
            let file = OpenOptions::new().read(true).open(save_file).unwrap();
            println!("{}", console::style("Loading savefile").bold());
            let res = read_savefile(&file).await?;
            modify_results(&server_info, res).await?
        }
    }

    Ok(())
}
