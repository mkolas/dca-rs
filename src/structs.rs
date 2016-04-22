use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Metadata {
    pub dca: DCAMetadata,
    pub song_info: SongMetadata,
    pub origin: OriginMetadata,
    pub opus: OpusMetadata,
    pub extra: BTreeMap<String, String>,
}

// Dca: DCAMetadata
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DCAMetadata {
    pub version: i8,
    pub tool: DCAToolMetadata,
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DCAToolMetadata {
    pub name: String,
    pub version: String,
    pub url: String,
    pub author: String,
}

// SongInfo: SongMetadata
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SongMetadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub comments: String,
    pub cover: String,
}

// Origin: OriginMetadata
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct OriginMetadata {
    pub source: String,
    pub bitrate: u32,
    pub channels: u32,
    pub encoding: String,
    pub url: String,
}

// Opus: OpusMetadata
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct OpusMetadata {
    pub bitrate: u32,
    pub sample_rate: u32,
    pub application: String,
    pub frame_size: u32,
    pub channels: u32,
}

// Extra: ExtraMetadata
// #[derive(Serialize, Deserialize, Debug)]
// pub struct ExtraMetadata;


// FFProbe structs
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FFProbeData {
    pub format: FFProbeFormat,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FFProbeFormat {
    pub filename: Option<String>,
    pub nb_streams: Option<i32>,
    pub nb_programs: Option<i32>,
    pub format_name: Option<String>,
    pub format_long_name: Option<String>,
    pub start_time: Option<String>,
    pub duration: Option<String>,
    pub size: Option<String>,
    #[serde(rename="bit_rate")]
    pub bitrate: Option<String>,
    pub probe_score: Option<i32>,
    pub tags: Option<FFProbeTags>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FFProbeTags {
    #[serde(rename="ARTIST")]
    pub artist: Option<String>,
    #[serde(rename="GENRE")]
    pub genre: Option<String>,
    #[serde(rename="TITLE")]
    pub title: Option<String>,
    #[serde(rename="ALBUM")]
    pub album: Option<String>,
    #[serde(rename="COMMENT")]
    pub comment: Option<String>,
}
