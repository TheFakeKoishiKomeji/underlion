use std::{error::Error, io::Read, ops::{Deref, DerefMut}, str::Utf8Error, string::FromUtf8Error};


use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_repr::Deserialize_repr;

pub const CF_BASE_URL: &str = "https://api.curseforge.com/v1/";

#[derive(Clone)]
pub struct Curseforge {
	key: String
}

impl Curseforge {
	pub fn new(key: String) -> Self {
		Self {
			key
		}
	}

	pub fn get_mod(&self, project_id: i32) -> Result<Mod, ApiError> {
		let query_url = format!("mods/{}", project_id);
		let result: DataResponse<Mod> = self.api_get(&query_url)?;
		Ok(result.data)
	}

	pub fn get_mods(&self, mod_ids: &[i32]) -> Result<Vec<Mod>, ApiError> {
		let body = GetModsBody{mod_ids};
		let result: DataResponse<Vec<Mod>> = self.api_post("mods", &body)?;
		Ok(result.data)
	}

	pub fn get_mod_file(&self, project_id: i32, file_id: i32) -> Result<File, ApiError> {
		let query_url = format!("mods/{}/files/{}", project_id, file_id);
		let result: DataResponse<File> = self.api_get(&query_url)?;
		Ok(result.data)
	}

	pub fn get_files(&self, file_ids: &[i32]) -> Result<Vec<File>, ApiError> {
		let body = GetFilesBody{file_ids};
		let result: DataResponse<Vec<File>> = self.api_post("mods/files", &body)?;
		Ok(result.data)
	}

	pub fn get_download_url(&self, project_id: i32, file_id: i32) -> Result<String, ApiError> {
		let query_url = format!("mods/{}/files/{}/download-url", project_id, file_id);
		let result: DataResponse<String> = self.api_get(&query_url)?;
		Ok(result.data)
	}

	fn api_get<T: DeserializeOwned>(&self, suburl: &str) -> Result<T, ApiError> {
		let query_url = format!("{}{}", CF_BASE_URL, suburl);
		let response = minreq::get(&query_url)
			.with_header("x-api-key", &self.key)
			.send()
			.ctx_error(&query_url)?;
		response.status_code.ctx_error(&query_url)?;
		let vec = response.into_bytes();
		let response = String::from_utf8(vec).ctx_error(&query_url)?;
		serde_json::from_str(&response).ctx_error((&query_url, &response))
	}

	fn api_post<T: DeserializeOwned, U: Serialize>(&self, suburl: &str, body: &U) -> Result<T, ApiError> {
		let query_url = format!("{}{}", CF_BASE_URL, suburl);
		let query_body = serde_json::to_string(body).ctx_error((&query_url, "N/A"))?;
		let response = minreq::post(&query_url)
			.with_header("x-api-key", &self.key)
			.send()
			.ctx_error(&query_url)?;
		response.status_code.ctx_error(&query_url)?;
		let vec = response.into_bytes();
		let response = String::from_utf8(vec).ctx_error(&query_url)?;
		serde_json::from_str(&response).ctx_error((&query_url, &response))
	}
}

#[derive(Deserialize, Clone, Debug)]
pub struct File {
	pub id: i32,
	#[serde(rename = "gameId")]
	pub game_id: i32,
	#[serde(rename = "modId")]
	pub mod_id: i32,
	#[serde(rename = "isAvailable")]
	pub is_available: bool,
	#[serde(rename = "displayName")]
	pub display_name: String,
	#[serde(rename = "fileName")]
	pub file_name: String,
	#[serde(rename = "releaseType")]
	pub release_type: FileReleaseType,
	#[serde(rename = "fileStatus")]
	pub file_status: FileStatus,
	pub hashes: Vec<FileHash>,
	#[serde(rename = "fileDate")]
	pub file_date: String,
	#[serde(rename = "fileLength")]
	pub file_length: i64,
	#[serde(rename = "downloadCount")]
	pub download_count: i64,
	#[serde(rename = "downloadUrl")]
	pub download_url: String,
	#[serde(rename = "gameVersions")]
	pub game_versions: Vec<String>,
	#[serde(rename = "sortableGameVersions")]
	pub sortable_game_versions: Vec<SortableGameVersion>,
	pub dependencies: Vec<FileDependency>,
	#[serde(rename = "exposeAsAlternative")]
	pub expose_as_alternative: Option<bool>,
	#[serde(rename = "parentProjectFileId")]
	pub parent_project_file_id: Option<i32>,
	#[serde(rename = "alternateFileId")]
	pub alternate_file_id: Option<i32>,
	#[serde(rename = "isServerPack")]
	pub is_server_pack: Option<bool>,
	#[serde(rename = "serverPackFileId")]
	pub server_pack_file_id: Option<i32>,
	#[serde(rename = "fileFingerprint")]
	pub file_fingerprint: i64,
	pub modules: Vec<FileModule>
}

#[derive(Clone, Deserialize, Debug)]
pub struct Mod {
	pub id: i32,
	#[serde(rename = "gameId")]
	pub game_id: i32,
	pub name: String,
	pub slug: String,
	pub links: ModLinks,
	pub summary: String,
	pub status: ModStatus,
	#[serde(rename = "downloadCount")]
	pub download_count: f64, // cf...
	#[serde(rename = "isFeatured")]
	pub is_featured: bool,
	#[serde(rename = "primaryCategoryId")]
	pub primary_category_id: i32,
	pub categories: Vec<Category>,
	#[serde(rename = "classId")]
	pub class_id: Option<i32>,
	pub authors: Vec<ModAuthor>,
	pub logo: Option<ModAsset>,
	pub screenshots: Vec<ModAsset>,
	#[serde(rename = "mainFileId")]
	pub main_file_id: i32,
	#[serde(rename = "latestFiles")]
	pub latest_files: Vec<File>,
	#[serde(rename = "latestFilesIndexes")]
	pub latest_files_indexes: Vec<FileIndex>,
	#[serde(rename = "dateCreated")]
	pub date_created: String,
	#[serde(rename = "dateModified")]
	pub date_modified: String,
	#[serde(rename = "dateReleased")]
	pub date_released: String,
	#[serde(rename = "allowModDistribution")]
	pub allow_mod_distribution: Option<bool>, // this name sucks
	#[serde(rename = "gamePopularityRank")]
	pub game_popularity_rank: i32,
	#[serde(rename = "isAvailable")]
	pub is_available: bool,
	#[serde(rename = "thumbsUpCount")]
	pub thumbs_up_count: Option<i32>
}

#[derive(Clone, Deserialize, Debug)]
pub struct Category {
	pub id: i32,
	#[serde(rename = "gameId")]
	pub game_id: i32,
	pub name: String,
	pub slug: String,
	pub url: String,
	#[serde(rename = "iconUrl")]
	pub icon_url: String,
	#[serde(rename = "dateModified")]
	pub date_modified: String,
	#[serde(rename = "is_class")]
	pub is_class: Option<bool>,
	#[serde(rename = "classId")]
	pub class_id: Option<i32>,
	#[serde(rename = "parentCategoryId")]
	pub parent_category_id: Option<i32>,
	#[serde(rename = "displayIndex")]
	pub display_index: Option<i32>
}

#[derive(Clone, Deserialize, Debug)]
pub struct FileIndex {
	#[serde(rename = "gameVersion")]
	pub game_version: String,
	#[serde(rename = "fileId")]
	pub file_id: i32,
	pub filename: String,
	#[serde(rename = "releaseType")]
	pub release_type: FileReleaseType,
	#[serde(rename = "gameVersionTypeId")]
	pub game_version_type_id: Option<i32>,
	#[serde(rename = "modLoader")]
	pub mod_loader: Option<ModLoaderType>
}

#[derive(Clone, Deserialize, Debug)]
pub struct ModAsset {
	pub id: i32,
	#[serde(rename = "modId")]
	pub mod_id: i32,
	pub title: String,
	pub description: String,
	#[serde(rename = "thumbnailUrl")]
	pub thumbnail_url: String,
	pub url: String
}

#[derive(Clone, Deserialize, Debug)]
pub struct ModAuthor {
	pub id: i32,
	pub name: String,
	pub url: String
}

#[derive(Clone, Deserialize, Debug)]
pub struct ModLinks {
	#[serde(rename = "websiteUrl")]
	pub website_url: Option<String>,
	#[serde(rename = "wikiUrl")]
	pub wiki_url: Option<String>,
	#[serde(rename = "issuesUrl")]
	pub issues_url: Option<String>,
	#[serde(rename = "sourceUrl")]
	pub source_url: Option<String>
}

#[derive(Clone, Deserialize, Debug)]
pub struct FileModule {
	pub name: String,
	pub fingerprint: i64
}

#[derive(Clone, Deserialize, Debug)]
pub struct FileDependency {
	#[serde(rename = "modId")]
	pub mod_id: i32,
	#[serde(rename = "relationType")]
	pub relation_type: FileRelationType
}

#[derive(Clone, Deserialize, Debug)]
pub struct SortableGameVersion {
	#[serde(rename = "gameVersionName")]
	pub game_version_name: String,
	#[serde(rename = "gameVersionPadded")]
	pub game_version_padded: String,
	#[serde(rename = "gameVersion")]
	pub game_version: String,
	#[serde(rename = "gameVersionReleaseDate")]
	pub game_version_release_date: String,
	#[serde(rename = "gameVersionTypeId")]
	pub game_version_type_id: Option<i32>
}

#[derive(Clone, Deserialize, Debug)]
pub struct FileHash {
	pub value: String,
	pub algo: HashAlgo
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize_repr)]
#[repr(u8)]
pub enum ModLoaderType {
	Any = 0,
	Forge = 1,
	Cauldron = 2,
	LiteLoader = 3,
	Fabric = 4
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize_repr)]
#[repr(u8)]
pub enum ModStatus {
	New = 1,
	ChangesRequired = 2,
	UnderSoftReview = 3,
	Approved = 4,
	Rejected = 5,
	ChangesMade = 6,
	Inactive = 7,
	Abandoned = 8,
	Deleted = 9,
	UnderReview = 10
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize_repr)]
#[repr(u8)]
pub enum FileRelationType {
	EmbeddedLibrary = 1,
	OptionalDependency = 2,
	RequiredDependency = 3,
	Tool = 4,
	Incompatible = 5,
	Include = 6
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize_repr)]
#[repr(u8)]
pub enum FileReleaseType {
	Release = 1,
	Beta = 2,
	Alpha = 3
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize_repr)]
#[repr(u8)]
pub enum FileStatus {
	Processing = 1,
	ChangesRequired = 2,
	UnderReview = 3,
	Approved = 4,
	Rejected = 5,
	MalwareDetected = 6,
	Deleted = 7,
	Archived = 8,
	Testing = 9,
	Released = 10,
	ReadyForReview = 11,
	Deprecated = 12,
	Baking = 13,
	AwaitingPublishing = 14,
	FailedPublishing = 15
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize_repr)]
#[repr(u8)]
pub enum HashAlgo {
	Sha1 = 1,
	Md5 = 2
}

#[derive(Deserialize, Clone, Copy)]
struct DataResponse<T> {
	data: T
}

#[derive(Serialize, Clone, Copy)]
struct GetModsBody<'a> {
	#[serde(rename = "modIds")]
	mod_ids: &'a [i32]
}

#[derive(Serialize, Clone, Copy)]
struct GetFilesBody<'a> {
	#[serde(rename = "fileIds")]
	file_ids: &'a [i32]
}

pub enum ApiError {
	HTTPError(minreq::Error, String),
	ResponseParseError(serde_json::Error, String, String),
	MalformedResponse(Utf8Error, String),
	BadHTTPResponse(String, i32),
	ForbiddenError(String),
	NotFoundError(String),
	ServerError(String),
	OtherError(String, Box<dyn Error>)
}

impl<T, U, V: ErrorContextualize<U>> ErrorContextualize<U> for Result<T, V> {
	type Contextualized = Result<T, V::Contextualized>;
	fn ctx_error(self, ctx: U) -> Self::Contextualized {
		match self {
			Ok(t) => Ok(t),
			Err(e) => Err(e.ctx_error(ctx))
		}
	}
}

impl ErrorContextualize<&str> for std::io::Error {
	type Contextualized = ApiError;
	fn ctx_error(self, ctx: &str) -> Self::Contextualized {
		ApiError::OtherError(ctx.to_string(), Box::new(self))
	}
}

impl ErrorContextualize<&str> for i32 {
	type Contextualized = Result<i32, ApiError>;
	fn ctx_error(self, ctx: &str) -> Self::Contextualized {
		if self / 100 == 2 {
			Ok(self)
		} else { 
			Err(match self {
				403 => ApiError::ForbiddenError(ctx.to_string()),
				404 => ApiError::NotFoundError(ctx.to_string()),
				500 => ApiError::ServerError(ctx.to_string()),
				_ => ApiError::BadHTTPResponse(ctx.to_string(), self)
			})
		}
	}
}

impl ErrorContextualize<&str> for minreq::Error {
	type Contextualized = ApiError;
	fn ctx_error(self, ctx: &str) -> Self::Contextualized {
		ApiError::HTTPError(self, ctx.to_string())
	}
}

impl ErrorContextualize<(&str, &str)> for serde_json::Error {
	type Contextualized = ApiError;
	fn ctx_error(self, ctx: (&str, &str)) -> Self::Contextualized {
		ApiError::ResponseParseError(self, ctx.0.to_string(), ctx.1.to_string())
	}
}

impl ErrorContextualize<&str> for Box<dyn Error> {
	type Contextualized = ApiError;
	fn ctx_error(self, ctx: &str) -> Self::Contextualized {
		ApiError::OtherError(ctx.to_string(), self)
	}
}

impl ErrorContextualize<&str> for Utf8Error {
	type Contextualized = ApiError;
	fn ctx_error(self, ctx: &str) -> Self::Contextualized {
		ApiError::MalformedResponse(self, ctx.to_string())
	}
}

impl ErrorContextualize<&str> for FromUtf8Error {
	type Contextualized = ApiError;
	fn ctx_error(self, ctx: &str) -> Self::Contextualized {
		self.utf8_error().ctx_error(ctx)
	}
}

impl ToString for ApiError {
	fn to_string(&self) -> String {
		match self {
			Self::HTTPError(error, url) => format!("Error contacting {}: {}", url, error),
			Self::ResponseParseError(error, url, response) => format!("Received invalid API data from {}: {}\nResponse:{}", url, error, response),
			Self::ForbiddenError(url) => format!("URL {} returned 403 Forbidden", url),
			Self::NotFoundError(url) => format!("URL {} returned 404 Not Found", url),
			Self::ServerError(url) => format!("URL {} returned 500 Server Error", url),
			Self::OtherError(url, err) => format!("Error requesting API at {}: {}", url, err),
			Self::BadHTTPResponse(url, code ) => format!("URL {} returned HTTP error {}", url, code),
			Self::MalformedResponse(error, url) => format!("URL {} responded with malformed UTF-8: {}", url, error)
		}
	}
}

pub trait ErrorStringify {
	type Stringified;
	fn stringify_error(self, prefix: &str) -> Self::Stringified;
}

impl<T, S: ToString> ErrorStringify for Result<T, S> {
	type Stringified = Result<T, String>;
	fn stringify_error(self, prefix: &str) -> Self::Stringified {
		match self {
			Ok(v) => Ok(v),
			Err(e) => Err(format!("{}: {}", prefix, e.to_string()))
		}
	}
}

trait ErrorContextualize<T> {
	type Contextualized;
	fn ctx_error(self, ctx: T) -> Self::Contextualized;
}
