//! Static asset data, custom dropped file models, and row structures for Finder (`SPEC-011`).

use std::path::PathBuf;

/// Static demo studio asset metadata.
pub struct StudioAssetRow {
    pub idx: usize,
    pub name: &'static str,
    pub codec: &'static str,
    pub duration: &'static str,
    pub size_mb: u32,
}

pub const STUDIO_ASSETS: [StudioAssetRow; 6] = [
    StudioAssetRow {
        idx: 0,
        name: "A001_C004_MainTitle.mov",
        codec: "Apple ProRes 422 HQ",
        duration: "00:01:24:12",
        size_mb: 1420,
    },
    StudioAssetRow {
        idx: 1,
        name: "A001_C009_VFX_Plate.mov",
        codec: "Apple ProRes 4444",
        duration: "00:00:42:08",
        size_mb: 2180,
    },
    StudioAssetRow {
        idx: 2,
        name: "Master_Mix_Stems_48k.wav",
        codec: "PCM 24-bit 48kHz",
        duration: "00:03:18:00",
        size_mb: 340,
    },
    StudioAssetRow {
        idx: 3,
        name: "Dailies_Proxy_Reel01.mp4",
        codec: "HEVC 10-bit HDR10",
        duration: "00:04:10:00",
        size_mb: 480,
    },
    StudioAssetRow {
        idx: 4,
        name: "Studio_ColorGrade_Rec709.cube",
        codec: "3D LUT Metal Shader",
        duration: "00:00:01:00",
        size_mb: 12,
    },
    StudioAssetRow {
        idx: 5,
        name: "Deliverable_Theatrical_4K.mov",
        codec: "Apple ProRes 422 HQ",
        duration: "00:12:45:00",
        size_mb: 8940,
    },
];

/// Custom file or folder entry loaded into State 4 Finder via disk browsing or OS Drag & Drop.
#[derive(Clone, Debug)]
pub struct FinderCustomEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size_mb: u32,
    pub codec: String,
    pub duration: String,
    pub rating: u32,
    pub tags: Vec<String>,
}

/// Unified display row for `MacTableView` in Finder browser.
#[derive(Clone, Debug)]
pub struct FinderDisplayRow {
    pub name: String,
    pub is_dir: bool,
    pub codec: String,
    pub duration: String,
    pub size_mb: u32,
    pub rating: u32,
    pub tags: Vec<String>,
    pub dir_path: Option<PathBuf>,
}

/// Build parent chain of directories for `MacPathControl`.
pub fn build_parent_chain(current_dir: &Option<PathBuf>) -> (Option<Vec<PathBuf>>, Vec<String>) {
    let dir_parents: Option<Vec<PathBuf>> = current_dir.as_ref().map(|cur_dir| {
        let mut chain = Vec::new();
        let mut cur: Option<&std::path::Path> = Some(cur_dir.as_path());
        while let Some(p) = cur {
            chain.push(p.to_path_buf());
            cur = p.parent();
        }
        chain.reverse();
        chain
    });

    let path_titles: Vec<String> = if let Some(ref parents) = dir_parents {
        parents
            .iter()
            .map(|p| {
                p.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| p.to_string_lossy().to_string())
            })
            .collect()
    } else {
        vec![
            "Macintosh HD".to_string(),
            "Projects".to_string(),
            "AppKit_Studio_v05".to_string(),
            "Master_Deliverables".to_string(),
        ]
    };

    (dir_parents, path_titles)
}

/// Collect display rows from either local directory or built-in studio assets.
pub fn collect_finder_rows(
    current_dir: &Option<PathBuf>,
    custom_entries: &[FinderCustomEntry],
    asset_ratings: &[u32],
    asset_tokens: &[Vec<String>],
) -> Vec<FinderDisplayRow> {
    if let Some(ref dir) = current_dir {
        let mut list = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let is_dir = path.is_dir();
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let size_mb = if is_dir {
                    0
                } else {
                    (entry.metadata().map(|m| m.len()).unwrap_or(0) / 1048576) as u32
                };
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or(if is_dir { "Folder" } else { "File" });
                list.push(FinderDisplayRow {
                    name,
                    is_dir,
                    codec: ext.to_uppercase(),
                    duration: if is_dir {
                        "Directory".into()
                    } else {
                        "--:--:--:--".into()
                    },
                    size_mb,
                    rating: 4,
                    tags: vec![ext.to_string()],
                    dir_path: if is_dir { Some(path) } else { None },
                });
            }
        }
        list
    } else {
        let mut list: Vec<FinderDisplayRow> = STUDIO_ASSETS
            .iter()
            .map(|r| FinderDisplayRow {
                name: r.name.to_string(),
                is_dir: false,
                codec: r.codec.to_string(),
                duration: r.duration.to_string(),
                size_mb: r.size_mb,
                rating: asset_ratings.get(r.idx).copied().unwrap_or(4),
                tags: asset_tokens.get(r.idx).cloned().unwrap_or_default(),
                dir_path: None,
            })
            .collect();
        for custom in custom_entries {
            list.push(FinderDisplayRow {
                name: custom.name.clone(),
                is_dir: custom.is_dir,
                codec: custom.codec.clone(),
                duration: custom.duration.clone(),
                size_mb: custom.size_mb,
                rating: custom.rating,
                tags: custom.tags.clone(),
                dir_path: Some(custom.path.clone()),
            });
        }
        list
    }
}

/// Filter and sort table rows.
pub fn filter_and_sort_rows(
    rows: Vec<FinderDisplayRow>,
    search_query: &str,
    sort_col: usize,
    sort_dir: egui_macview::MacSortDirection,
) -> Vec<FinderDisplayRow> {
    let q = search_query.trim().to_lowercase();
    let mut visible: Vec<FinderDisplayRow> = rows
        .into_iter()
        .filter(|r| {
            if q.is_empty() {
                return true;
            }
            let tags_match = r.tags.iter().any(|tag| tag.to_lowercase().contains(&q));
            r.name.to_lowercase().contains(&q) || r.codec.to_lowercase().contains(&q) || tags_match
        })
        .collect();

    visible.sort_by(|a, b| {
        let ord = match sort_col {
            0 => a.name.cmp(&b.name),
            1 => a.codec.cmp(&b.codec),
            2 => a.duration.cmp(&b.duration),
            3 => a.size_mb.cmp(&b.size_mb),
            _ => a.rating.cmp(&b.rating),
        };
        match sort_dir {
            egui_macview::MacSortDirection::Ascending => ord,
            egui_macview::MacSortDirection::Descending => ord.reverse(),
        }
    });

    visible
}
