use serde::Deserialize;

#[derive(Deserialize)]
pub struct Manifest {
	pub minecraft: MinecraftInfo,
	#[serde(rename = "manifestType")]
	pub manifest_type: String,
	pub overrides: String,
	#[serde(rename = "manifestVersion")]
	pub manifest_version: u32,
	pub version: String,
	pub author: String,
	pub name: String,
	pub files: Vec<FileInfo>
}

#[derive(Deserialize)]
pub struct MinecraftInfo {
	pub version: String,
	#[serde(rename = "modLoaders")]
	pub mod_loaders: Vec<LoaderInfo>
}

#[derive(Deserialize)]
pub struct LoaderInfo {
	pub id: String,
	pub primary: bool
}

#[derive(Deserialize)]
pub struct FileInfo {
	#[serde(rename = "projectID")]
	pub project_id: i32,
	#[serde(rename = "fileID")]
	pub file_id: i32, 
	pub required: bool
}

