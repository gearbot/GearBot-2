use std::ops::Range;

pub struct ClusterInfo {
    pub cluster_id: u16,
    pub shards: Range<u64>,
    pub cluster_identifier: String,
    pub total_shards: u64,
}
