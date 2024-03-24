use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    #[serde(skip_serializing)]
    pub id: Option<String>,
    pub album_name: String,
    pub assets: Vec<Asset>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AlbumSaveFile {
    pub album_name: String,
    pub assets: Vec<String>,
}

impl AlbumSaveFile {
    pub fn from_album(album: Album) -> Self {
        AlbumSaveFile {
            album_name: album.album_name,
            assets: album
                .assets
                .into_iter()
                .map(|x| x.original_file_name.unwrap())
                .collect(),
        }
    }

    pub fn to_album(self) -> Album {
        Album {
            id: None,
            album_name: self.album_name,
            assets: self
                .assets
                .into_iter()
                .map(|x| Asset {
                    original_file_name: Some(x),
                    id: None,
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub original_file_name: Option<String>,
    #[serde(skip_serializing)]
    pub id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FavouriteCollection {
    pub favorites: Vec<Asset>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FavouriteCollectionSaveFile {
    pub favorites: Vec<String>,
}

impl FavouriteCollectionSaveFile {
    pub fn new() -> Self {
        FavouriteCollectionSaveFile {
            favorites: Vec::new(),
        }
    }
    pub fn from_fav_collection(fav: FavouriteCollection) -> Self {
        FavouriteCollectionSaveFile {
            favorites: fav
                .favorites
                .into_iter()
                .map(|x| x.original_file_name.unwrap())
                .collect(),
        }
    }

    pub fn to_fav_collection(self) -> FavouriteCollection {
        FavouriteCollection {
            favorites: self
                .favorites
                .into_iter()
                .map(|x| Asset {
                    original_file_name: Some(x),
                    id: None,
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SaveFile {
    FavouriteCollection,
    Album,
}
