use crate::source_graph::{EnergyClass, SourceGraph, SourceMapPeakClass};

use super::{
    SOURCE_MAP_BLOCKS, SOURCE_MAP_WIDTH, source_map_column_end_seconds,
    source_map_column_midpoint_seconds, source_map_column_start_seconds, source_map_energy_block,
};

pub(super) fn source_map_energy_row(graph: &SourceGraph) -> String {
    if !graph.source_map.buckets.is_empty() {
        return (0..SOURCE_MAP_WIDTH)
            .map(|column| {
                let time = source_map_column_midpoint_seconds(graph, column);
                graph
                    .source_map
                    .buckets
                    .iter()
                    .find(|bucket| time >= bucket.start_seconds && time < bucket.end_seconds)
                    .map_or(SOURCE_MAP_BLOCKS[0], |bucket| {
                        source_map_energy_block(bucket.energy_class)
                    })
            })
            .collect();
    }

    (0..SOURCE_MAP_WIDTH)
        .map(|column| {
            let time = source_map_column_midpoint_seconds(graph, column);
            let energy = graph
                .sections
                .iter()
                .find(|section| time >= section.start_seconds && time < section.end_seconds)
                .map_or(EnergyClass::Unknown, |section| section.energy_class);
            source_map_energy_block(energy)
        })
        .collect()
}

pub(super) fn source_map_peak_row(graph: &SourceGraph) -> String {
    if !graph.source_map.buckets.is_empty() {
        return (0..SOURCE_MAP_WIDTH)
            .map(|column| {
                let start = source_map_column_start_seconds(graph, column);
                let end = source_map_column_end_seconds(graph, column);
                source_map_bucket_peak_block(graph, start, end)
            })
            .collect();
    }

    (0..SOURCE_MAP_WIDTH)
        .map(|column| {
            let start = source_map_column_start_seconds(graph, column);
            let end = source_map_column_end_seconds(graph, column);
            if source_map_bucket_has_anchor(graph, start, end) {
                SOURCE_MAP_BLOCKS[4]
            } else if source_map_bucket_has_asset(graph, start, end) {
                SOURCE_MAP_BLOCKS[3]
            } else {
                '.'
            }
        })
        .collect()
}

fn source_map_bucket_has_anchor(graph: &SourceGraph, start: f32, end: f32) -> bool {
    graph
        .timing
        .primary_hypothesis()
        .into_iter()
        .flat_map(|hypothesis| hypothesis.anchors.iter())
        .any(|anchor| anchor.time_seconds >= start && anchor.time_seconds < end)
}

fn source_map_bucket_has_asset(graph: &SourceGraph, start: f32, end: f32) -> bool {
    graph
        .assets
        .iter()
        .any(|asset| asset.start_seconds < end && asset.end_seconds > start)
}

fn source_map_bucket_peak_block(graph: &SourceGraph, start: f32, end: f32) -> char {
    graph
        .source_map
        .buckets
        .iter()
        .filter(|bucket| bucket.end_seconds > start && bucket.start_seconds < end)
        .map(|bucket| match bucket.peak_class {
            SourceMapPeakClass::StrongTransient => SOURCE_MAP_BLOCKS[4],
            SourceMapPeakClass::Transient => SOURCE_MAP_BLOCKS[3],
            SourceMapPeakClass::None => '.',
        })
        .max_by_key(|block| match block {
            '█' => 2,
            '▇' => 1,
            _ => 0,
        })
        .unwrap_or('.')
}
