use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct AudioStreamUrl {
    pub seek_param: String,
    pub stream_type: String,
    pub codec: String,
    pub time_length: f64,
    pub sample_rate: i64,
    pub audio_mode: String,
    pub bitrate: i64,
    pub file_size: i64,
    pub md5: String,
    pub audio_url: String,
    pub backup_url: Vec<String>,
}
