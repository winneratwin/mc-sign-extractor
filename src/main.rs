use clap::Parser;
use std::path::{Path, PathBuf};
use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use flate2::read::{ZlibDecoder, GzDecoder};
use serde::{Deserialize, Serialize};
//use nbt;


#[derive(Parser,Debug)]
#[command(author, version, about, long_about)]
struct Opts {
	/// minecraft save folder
	#[clap(short, long)]
	save: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct LevelDat {
	#[serde(rename = "Data")]
	data:LevelDatData
}
#[derive(Debug, Serialize, Deserialize)]
struct LevelDatData {
	#[serde(rename = "Version")]
	version: Option<LevelDatDataVersion>,
	#[serde(rename = "version")]
	old_version: i32
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct LevelDatDataVersion {
	#[serde(rename = "Id")]
	id: i32,
	#[serde(rename = "Name")]
	name: String,
	#[serde(rename = "Snapshot")]
	snapshot: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Chunk {
	#[serde(rename = "Level")]
	level: ChunkLevel
}
#[derive(Debug, Serialize, Deserialize)]
struct ChunkLevel {
	#[serde(rename = "TileEntities")]
	tile_entities: Vec<ChunkLevelTileEntities>
}
#[derive(Debug, Serialize, Deserialize)]
struct ChunkLevelTileEntities {
	#[serde(rename = "id")]
	id: String,
	#[serde(rename = "x")]
	x: i32,
	#[serde(rename = "y")]
	y: i32,
	#[serde(rename = "z")]
	z: i32,
	// Text1-4 are for signs
	#[serde(rename = "Text1")]
	text1: Option<String>,
	#[serde(rename = "Text2")]
	text2: Option<String>,
	#[serde(rename = "Text3")]
	text3: Option<String>,
	#[serde(rename = "Text4")]
	text4: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Chunk1_13 {
	#[serde(rename = "block_entities")]
	block_entities: Vec<ChunkLevelTileEntities>
}


fn main() {
	let opts: Opts = Opts::parse();

	// check if save folder exists
	let save_path = Path::new(&opts.save);
	if !save_path.exists() {
		println!("save folder does not exist");
		return;
	}
	// check if save folder is a directory
	if !save_path.is_dir() {
		println!("save folder is not a directory");
		return;
	}

	// get save version
	let version_path = save_path.join("level.dat");
	if !version_path.exists() {
		println!("save version does not exist");
		return;
	}
	let version_file = File::open(version_path).expect("failed to open file");
	//println!("{:#?}",nbt::Blob::from_reader(&mut GzDecoder::new(version_file)).expect("failed to read nbt"));
	//return;
	let version_nbt: LevelDat = nbt::from_reader(&mut GzDecoder::new(version_file)).expect("failed to read nbt");

	// if Version is None then we are using an old version of minecraft
	// fallback to old version
	let version = match version_nbt.data.version {
		Some(version) => version,
		None => {
			LevelDatDataVersion {
				id: version_nbt.data.old_version,
				name: "old".to_string(),
				snapshot: false
			}
		}
	};

	
	println!("world_version: {} id: {}", version.name, version.id);


	// get all files in region folder
	let region_path = save_path.join("region");
	let region_files = region_path.read_dir().unwrap();
	let mut signs:Vec<ChunkLevelTileEntities> = Vec::new();

	// get number of threads
	let num_threads = num_cpus::get();

	// create thread pool
	let pool = threadpool::Builder::new().num_threads(num_threads).build();

	// create a channel to send the results from the threads
	let (tx, rx) = std::sync::mpsc::channel();

	let mut number_of_files = 0;
	for file in region_files {
		let file = file.unwrap();
		let file_path = file.path();

		// clone the sender
		let thread_tx = tx.clone();
		let thread_version = version.clone();
		pool.execute(move || {
			// extract signs from mca file
			thread_tx.send(extract_signs_from_mca(file_path, thread_version)).unwrap();
		});
		number_of_files += 1;
	}
	pool.join();

	// collect all the results from the threads
	rx.iter().take(number_of_files).for_each(|signs_from_thread| {
		signs.extend(signs_from_thread);
	});

	// sort signs by x then z
	signs.sort_by(|a, b| {
		a.x.cmp(&b.x).then(a.z.cmp(&b.z)).then(a.y.cmp(&b.y))
	});
	

	// if version is old then the text is raw but if it is newer then it is json
	// the json is in the format {"text":"text"} with an optional "extra" field
	// that contains an array of more json objects
	
	#[derive(Debug, Serialize, Deserialize)]
	struct SignExtra {
		text: String, // text of the json object
		color: Option<String>, // color of the text
		bold: Option<bool>, // if true then the text is bold
		italic: Option<bool>, // if true then the text is italic
		underlined: Option<bool>, // if true then the text is underlined
		strikethrough: Option<bool>, // if true then the text is crossed out
		obfuscated: Option<bool>, // if true then the text is randomly scrambled every time it is displayed
	}
	#[derive(Debug, Serialize, Deserialize)]
	struct SignText {
		text: String,
		extra: Option<Vec<SignExtra>>,
	}

	for sign in signs {
		// print xyz coordinates
		println!("---------- location: {},{},{} ----------", sign.x, sign.y, sign.z);
		// print text all text fields
		// all text fields exist since we only extract signs
		if version.name != "old".to_owned() {
			// convert sign text from json to struct
			let sign_text_1: SignText = serde_json::from_str(&sign.text1.unwrap()).unwrap();
			
			// if extra exists then combine all the text fields
			if let Some(extra) = sign_text_1.extra {
				let mut text = sign_text_1.text;
				for extra in extra {
					text.push_str(&extra.text);
				}
				println!("text: {}", text);
			} else {
				println!("text: {}", sign_text_1.text);
			}

			// repeat for all text fields
			
			let sign_text_2: SignText = serde_json::from_str(&sign.text2.unwrap()).unwrap();
			if let Some(extra) = sign_text_2.extra {
				let mut text = sign_text_2.text;
				for extra in extra {
					text.push_str(&extra.text);
				}
				println!("text: {}", text);
			} else {
				println!("text: {}", sign_text_2.text);
			}

			let sign_text_3: SignText = serde_json::from_str(&sign.text3.unwrap()).unwrap();
			if let Some(extra) = sign_text_3.extra {
				let mut text = sign_text_3.text;
				for extra in extra {
					text.push_str(&extra.text);
				}
				println!("text: {}", text);
			} else {
				println!("text: {}", sign_text_3.text);
			}

			let sign_text_4: SignText = serde_json::from_str(&sign.text4.unwrap()).unwrap();
			if let Some(extra) = sign_text_4.extra {
				let mut text = sign_text_4.text;
				for extra in extra {
					text.push_str(&extra.text);
				}
				println!("text: {}", text);
			} else {
				println!("text: {}", sign_text_4.text);
			}

		} else {
			// if version is old then the text is raw
			println!("text1: {}", sign.text1.unwrap());
			println!("text2: {}", sign.text2.unwrap());
			println!("text3: {}", sign.text3.unwrap());
			println!("text4: {}", sign.text4.unwrap());
		}
	}


    eprintln!("done!");
}

fn extract_signs_from_mca(file_path:PathBuf, version:LevelDatDataVersion) -> Vec<ChunkLevelTileEntities> {
	let mut signs:Vec<ChunkLevelTileEntities> = Vec::new();
	let file_name = file_path.file_name().unwrap().to_str().unwrap();

	// check if file name matches regex
	let re: Regex = Regex::new(r"r\.(?P<rx>-?\d+)\.(?P<ry>-?\d+)\.mca").expect("invalid regex");
	let caps = match re.captures(file_name){
		Some(caps) => caps,
		None => return signs,
	};

	// convert to i32
	let rx = caps.name("rx").unwrap().as_str().parse::<i32>().unwrap();
	let ry = caps.name("ry").unwrap().as_str().parse::<i32>().unwrap();
	// print chunk coordinates using std err to not mess up the output when piping to a file
	eprintln!("---------- reading chunk: {}, {} ----------", rx, ry);

	// check if file is not empty/corrupted
	let metadata = std::fs::metadata(file_path.clone()).expect("failed to get metadata");
	if metadata.len() == 0 {
		return signs;
	}


	// open file
	let mut region_file = File::open(file_path).expect("failed to open file");

	// read headers
	for x in 0..32 {
		for z in 0..32 {
			// seek to header
			let offset = (x + z * 32) * 4;
			region_file.seek(std::io::SeekFrom::Start(offset as u64)).expect("failed to seek");

			// read 4 bytes
			let mut header = [0; 4];
			region_file.read_exact(&mut header).expect("failed to read header");

			// first 3 bytes are offset
			// last byte is number of 4KiB sectors
			let offset = (header[0] as u32) << 16 | (header[1] as u32) << 8 | (header[2] as u32);
			let sectors = header[3] as u32;

			// check if chunk is present
			if sectors == 0 {
				continue;
			}

			// seek to chunk
			let chunk_offset = offset as u64 * 4096;
			region_file.seek(std::io::SeekFrom::Start(chunk_offset)).expect("failed to seek");

			// read chunk length of remaining chunk bytes
			let mut length = [0; 4];
			region_file.read_exact(&mut length).expect("failed to read length");

			// convert to big endian
			let length = u32::from_be_bytes(length);

			// get compression type (5th byte)
			// 1 = gzip
			// 2 = zlib
			// 3 = uncompressed
			let mut compression_type = [0; 1];
			region_file.read_exact(&mut compression_type).expect("failed to read compression type");

			// if compression type is zlib read the chunk
			if compression_type[0] != 2 {
				println!("unsupported compression type: {}", compression_type[0]);
				continue;
			}

			let mut chunk = vec![0; (length-1) as usize];
			region_file.read_exact(&mut chunk).expect("failed to read chunk");

			// if version is less or equal to 1.17.1 (2730) due to 1.18 changing the nbt format of chunks because
			// of the new height limit
			//println!("{:?}", nbt::Blob::from_reader(&mut ZlibDecoder::new(&chunk[..])));

			if version.id > 2730 && version.name != "old".to_owned() {
				let nbt_data: Chunk1_13 = match nbt::from_reader(&mut ZlibDecoder::new(&chunk[..])) {
					Ok(nbt_data) => nbt_data,
					Err(e) => {
						// print error and chunk coordinates
						eprintln!("failed to read nbt in chunk: {}, {} with error {}", rx, ry, e);
						//println!("data: {:?}", nbt::Blob::from_reader(&mut ZlibDecoder::new(&chunk[..])));
						continue;
					}
				};
	
				for block_entity in nbt_data.block_entities {
					// if block entity is a sign
					if block_entity.id.ends_with("sign") {
						signs.push(block_entity);
					}
				}
			} else {
				let nbt_data: Chunk = match nbt::from_reader(&mut ZlibDecoder::new(&chunk[..])) {
					Ok(nbt_data) => nbt_data,
					Err(e) => {
						// print error and chunk coordinates
						eprintln!("failed to read nbt in chunk: {}, {} with error {}", rx, ry, e);
						continue;
					}
				};
				// iterate over tile entities
				for tile_entity in nbt_data.level.tile_entities {
					// if tile entity is a sign
					// convert to lowercase because somewhere between 1.12.2 and 1.9.4 the id changed from "minecraft:sign" to "Sign"
					if tile_entity.id.to_lowercase().ends_with("sign") {
						signs.push(tile_entity);
					}
				}
			}
		}
	}

	return signs;
}