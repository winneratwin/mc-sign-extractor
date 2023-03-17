
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct LevelDat {
	#[serde(rename = "Data")]
	pub data:LevelDatData
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LevelDatData {
	#[serde(rename = "Version")]
	pub version: Option<LevelDatDataVersion>,
	#[serde(rename = "version")]
	pub old_version: i32
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LevelDatDataVersion {
	#[serde(rename = "Id")]
	pub id: i32,
	#[serde(rename = "Name")]
	pub name: String,
	#[serde(rename = "Snapshot")]
	pub snapshot: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
	#[serde(rename = "Level")]
	pub level: ChunkLevel
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkLevel {
	#[serde(rename = "TileEntities")]
	pub tile_entities: Vec<ChunkLevelTileEntities>,
	#[serde(rename = "Entities")]
	pub entities: Vec<Entity>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
	#[serde(rename = "id")]
	id: String,
	#[serde(rename = "Pos")]
	pub pos: Vec<f64>,
	#[serde(rename = "Item")]
	pub item: Option<Item>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkLevelTileEntities {
	#[serde(rename = "id")]
	pub id: String,
	#[serde(rename = "x")]
	pub x: i32,
	#[serde(rename = "y")]
	pub y: i32,
	#[serde(rename = "z")]
	pub z: i32,
	// Text1-4 are for signs
	#[serde(rename = "Text1")]
	pub text1: Option<String>,
	#[serde(rename = "Text2")]
	pub text2: Option<String>,
	#[serde(rename = "Text3")]
	pub text3: Option<String>,
	#[serde(rename = "Text4")]
	pub text4: Option<String>,
	#[serde(rename = "Items")]
	pub items: Option<Vec<Item>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
	#[serde(rename = "id")]
	pub id: String,
	#[serde(rename = "Slot")]
	slot: Option<i8>,
	#[serde(rename = "Count")]
	count: i8,
	#[serde(rename = "tag")]
	pub tag: Option<Book>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk1_18 {
	#[serde(rename = "block_entities")]
	pub block_entities: Vec<ChunkLevelTileEntities>
}

// 1.17 remove Entities from chunk and put it in a separate file
// and also moves TileEntities to Level
#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk1_17 {
	#[serde(rename = "Level")]
	pub level: Chunk1_17Level
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk1_17Level {
	#[serde(rename = "TileEntities")]
	pub block_entities: Vec<ChunkLevelTileEntities>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SignExtra {
	pub text: String, // text of the json object
	color: Option<String>, // color of the text
	bold: Option<bool>, // if true then the text is bold
	italic: Option<bool>, // if true then the text is italic
	underlined: Option<bool>, // if true then the text is underlined
	strikethrough: Option<bool>, // if true then the text is crossed out
	obfuscated: Option<bool>, // if true then the text is randomly scrambled every time it is displayed
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignText {
	pub text: String,
	pub extra: Option<Vec<SignExtra>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Book {
	#[serde(rename = "pages")]
	pub pages: Option<Vec<String>>,
	#[serde(rename = "title")]
	pub title: Option<String>,
	#[serde(rename = "author")]
	pub author: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookWithPos {
	pub book: Book,
	pub x: i32,
	pub y: i32,
	pub z: i32,
}
