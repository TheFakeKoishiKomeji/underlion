mod threadpool;
mod manifest;
mod api;

use std::{fs::{self, File}, io::{Cursor, Read, Seek, Write}, path::{Path, PathBuf}};

use api::Curseforge;
use manifest::*;
use regex::Regex;
use threadpool::{BranchedExecutor, ThreadPool};
use clap::{Parser, Subcommand};
use zip::{ZipArchive, read::ZipFile};

use crate::api::ErrorStringify;

const DEFAULT_KEY_PATH: &str = ".cfkey";
const MANIFEST_NAME: &str = "manifest.json";
const KEY_GRAB_LOCATION: &str = "dist/desktop/desktop.js";

#[derive(Parser, Debug)]
#[clap()]
struct Args {
	#[clap(subcommand)]
	action: Action
}

fn main() {
	let args = Args::parse();
	if let Err(s) = run_command(args) {
		println!("{}", s);
	}
}

fn run_command(args: Args) -> Result<(), String> {
	match args.action {
		Action::Install {
			pack_zip,
			install_to,
			key_file,
			key,
			parallel
		} => {
			let pack_name = pack_zip.file_stem().try_expect("No pack filename given?")?;
			let install_to_path = path_or(&install_to, Path::new(pack_name)).to_path_buf();
			let mut mods_folder = install_to_path.clone();
			mods_folder.push("mods");

			let mut pack = try_open_zip(&pack_zip)?;
			
			let manifest = {
				// borrowck throws a fit about manifest_file if I don't limit its scope
				let mut manifest_file = try_read_zip_entry(&mut pack, MANIFEST_NAME)?;
				try_read_manifest(&mut manifest_file)?
			};

			try_mkdir(&install_to_path)?;
			try_mkdir(&mods_folder)?;

			let key = get_key(key, &key_file)?.trim().to_string();

			let threads = parallel.unwrap_or(1);
			let mv_mods = mods_folder.clone();
			let exec = if threads > 1 {
				let pool: ThreadPool<FileInfo> = ThreadPool::new::<_, Curseforge, _>(threads, 
					move |file, cf| {
						if let Err(e) = download(&file, cf, &mv_mods) {
							println!("{}", e);
						}
					},
					move || {
						Curseforge::new(key.clone())
					}
				);
				BranchedExecutor::Pooled(pool)
			} else {
				let cf = Curseforge::new(key);
				BranchedExecutor::ThisThread(Box::new(move |file| {
					if let Err(e) = download(&file, &cf, &mv_mods) {
						println!("{}", e);
					}
				}))
			};
			if let Ok(wait) = exec.exec(manifest.files) {
				wait.wait();
			} else {
				println!("Threadpool error -- could not download files.")
			}
			
			// extract overrides
			let mut fnames = Vec::new();
			for fname in pack.file_names() {
				if let Some(s) = fname.split('/').next() {
					if s == manifest.overrides {
						fnames.push(fname.to_string());
					}
				}
			}
			
			for fname in fnames {
				let mut entry = try_read_zip_entry(&mut pack, &fname)?;
				if entry.is_file() {
					let name = entry
						.enclosed_name()
						.try_expect("Could not properly format name for writing to filesystem")?
						.strip_prefix(&format!("{}/", manifest.overrides))
						.stringify_error("Error converting path")?
						.to_owned();
					let mut ext_path = install_to_path.clone();
					ext_path.push(name);
					if let Some(p) = ext_path.parent() {
						try_mkdir(&p)?;
					}
					let mut data = Vec::new();
					entry.read_to_end(&mut data).stringify_error("Could not read zip data")?;
					let mut file = try_open_write(&ext_path)?;
					file.write_all(&data).stringify_error("Error writing override data to disk")?;
				}
			}
		},
		Action::FindBad {
			pack_zip,
			key_file,
			key
		} => {
			let mut pack = try_open_zip(&pack_zip)?;
			let mut manifest_file = try_read_zip_entry(&mut pack, MANIFEST_NAME)?;
			let manifest = try_read_manifest(&mut manifest_file)?;
			let key = get_key(key, &key_file)?.trim().to_string();

			let mut mod_ids = Vec::new();
			let cf = Curseforge::new(key);
			for file in manifest.files {
				mod_ids.push(file.project_id);
			}
			let mods = cf.get_mods(&mod_ids).stringify_error("Error getting mod metadata")?;
			for m in mods {
				if let Some(false) = m.allow_mod_distribution {
					println!("Non-downloadable mod: {} ({})", m.name, m.slug);
				}
			}
		},
		Action::GrabKey {
			cf_version,
			cf_url
		} => {
			let url = if let Some(url) = cf_url {
				url
			} else {
				let ver = if let Some(k) = cf_version.as_ref() {
					k
				} else {
					"0.196.1.11"
				};
				format!("https://appsdl-overwolf-com.akamaized.net/prod/apps/cchhcaiapeikjbdbpfplgmpobbcdkdaphclbmkbj/{ver}/app.opk")
			};
			grab_key(&url)?;
		}
	}
	Ok(())
}

fn grab_key(cf_url: &str) -> Result<(), String> {
	let response = minreq::get(cf_url)
		.send()
		.stringify_error("Error making request to CF download")?;
	let bytes = response.into_bytes();

	let mut cf_zip = ZipArchive::new(Cursor::new(bytes)).stringify_error("Error reading CF download as zip")?;
	let mut file_with_token = try_read_zip_entry(&mut cf_zip, KEY_GRAB_LOCATION)?;
	let mut loaded_file_with_token = String::new();
	file_with_token.read_to_string(&mut loaded_file_with_token).stringify_error("Error loading file containing token as string.")?;
	let key = read_key_from_str(&loaded_file_with_token)?;

	let mut key_file = try_open_write(Path::new(DEFAULT_KEY_PATH))?;
	write!(key_file, "{}", key).stringify_error("Error writing to key file")?;

	Ok(())
}

fn read_key_from_str(key_file: &str) -> Result<String, String> {
	let pat = Regex::new("cfCoreApiKey\":\"(.*?)\"").unwrap();
	let key = pat.captures(key_file).try_expect("Key not found in file -- likely changed.")?[1].to_string();
	Ok(key)
}

fn download(file: &FileInfo, cf: &Curseforge, mods_dir: &Path) -> Result<(), String> {
	let url = cf.get_download_url(file.project_id, file.file_id).stringify_error("Error fetching download URL")?;
	println!("Downloading {}", &url);
	let response = match minreq::get(&url).send() {
		Ok(r) => Ok(r),
		Err(e) => Err(format!("Error downloading file {}: {}", url, e))
	}?;
	if response.status_code / 100 != 2 {
		Err(format!("HTTP Error downloading file {}: {}", url, response.status_code))
	} else {
		let mut path = mods_dir.to_path_buf();
		let filename = url.split("/").last().try_expect("Error getting filename, does URL have no slashes?")?;
		path.push(&filename);
		let mut file = try_open_write(&path)?;
		if let Err(e) = file.write_all(response.as_bytes()) {
			Err(format!("Error writing downloaded file {}: {}", filename, e))
		} else {
			Ok(())
		}
	}
}


#[derive(Subcommand, Debug)]
enum Action {
	/// Installs a curseforge pack.
	Install {
		/// Curseforge pack zip to use.
		pack_zip: PathBuf,
		/// Directory to install to
		install_to: Option<PathBuf>,

		/// Use a different file as the CF API key
		#[clap(short = 'f', long)]
		key_file: Option<PathBuf>,

		/// Use a different CF API key. (Overrides key_file.)
		#[clap(short, long)]
		key: Option<String>,

		/// Use parallel threads of provided count for downloads.
		#[clap(short, long)]
		parallel: Option<u32>
	},
	/// Finds mods in a curseforge pack which have disabled downloads.
	FindBad {
		/// Curseforge pack zip to check
		pack_zip: PathBuf,

		/// Use a different file as the CF API key
		#[clap(short = 'f', long)]
		key_file: Option<PathBuf>,

		/// Use a different CF API key. (Overrides key_file.)
		#[clap(short, long)]
		key: Option<String>,
	},
	/// Grabs the internal curseforge key that allows downloading even mods with downloads disabled.
	GrabKey {
		/// Use a different version of the CF Overwolf extension.
		#[clap(short = 'v', long)]
		cf_version: Option<String>,

		/// Use an alternate URL to download the CF Overwolf extension. (Overrides cf_version)
		#[clap(short = 'u', long)]
		cf_url: Option<String>
	},
}

fn path_or<'a>(p1: &'a Option<PathBuf>, default: &'a Path) -> &'a Path {
	match p1.as_ref() {
		Some(p) => p,
		None => default
	}
}

fn get_key(input_key: Option<String>, input_file: &Option<PathBuf>) -> Result<String, String> {
	match (input_key, input_file) {
		(Some(k), _) => Ok(k),
		(None, Some(f)) => {
			try_load_file(f)
		},
		_ => {
			if let Ok(s) = try_load_file(Path::new(DEFAULT_KEY_PATH)) {
				Ok(s)
			} else {
				Err("No key or key file provided, and default key file does not exist!".into())
			}
		}
	}
}

fn try_load_file(file: &Path) -> Result<String, String> {
	match fs::read_to_string(file) {
		Ok(f) => Ok(f),
		Err(e) => Err(format!("Error opening file {:?}: {}", file, e))
	}
}

trait TryExpect<T> {
	fn try_expect(self, msg: &str) -> Result<T, String>;
}

impl<T> TryExpect<T> for Option<T> {
	fn try_expect(self, msg: &str) -> Result<T, String> {
		match self {
			Some(t) => Ok(t),
			None => Err(msg.into())
		}
	}
}

fn try_mkdir(path: &Path) -> Result<(), String> {
	if let Err(e) = fs::create_dir_all(path) {
		Err(format!("Error creating directory {:?}: {}", path, e))
	} else {
		Ok(())
	}
}

fn try_open(path: &Path) -> Result<File, String> {
	match File::open(path) {
		Ok(f) => Ok(f),
		Err(e) => Err(format!("Error opening file {:?}: {}", path, e))
	}
}

fn try_open_write(path: &Path) -> Result<File, String> {
	match File::create(path) {
		Ok(f) => Ok(f),
		Err(e) => Err(format!("Error opening file {:?}: {}", path, e))
	}
}

fn try_open_zip(path: &Path) -> Result<ZipArchive<File>, String> {
	let file = try_open(path)?;
	match ZipArchive::new(file) {
		Ok(z) => Ok(z),
		Err(e) => Err(format!("Error opening pack zip {:?}: {}", path, e))
	}
}

fn try_read_zip_entry<'a, T: Read + Seek>(zip: &'a mut ZipArchive<T>, loc: &str) -> Result<ZipFile<'a>, String> {
	match zip.by_name(loc) {
		Ok(f) => Ok(f),
		Err(e) => Err(format!("Error reading zip entry {}: {}", loc, e))
	}
}

fn try_read_manifest<T: Read>(t: &mut T) -> Result<Manifest, String> {
	match serde_json::from_reader(t) {
		Ok(m) => Ok(m),
		Err(e) => Err(format!("Error parsing pack manifest: {}", e))
	}
}
