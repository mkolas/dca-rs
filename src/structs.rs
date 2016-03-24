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
    pub filename: String,
    pub nb_streams: i32,
    pub nb_programs: i32,
    pub format_name: String,
    pub format_long_name: String,
    pub start_time: String,
    pub duration: String,
    pub size: String,
    #[serde(rename="bit_rate")]
    pub bitrate: String,
    pub probe_score: i32,
    pub tags: FFProbeTags,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FFProbeTags {
    pub date: Option<String>,
    pub track: Option<String>,
    pub artist: Option<String>,
    pub genre: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
    pub compilation: Option<String>,
}
