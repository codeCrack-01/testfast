use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ast_engine::extractor::extract_skeleton;
use crate::ast_engine::skeleton::FileSkeleton;

const CACHE_DIR: &str = ".coderag";
const CACHE_FILE: &str = ".coderag/cache.json";

/// A single cache entry — maps a file's content hash to its skeleton.
#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    hash: String,
    skeleton: FileSkeleton,
}

/// On-disk cache: file path → cached entry.
#[derive(Debug, Serialize, Deserialize)]
pub struct SkeletonCache {
    files: HashMap<String, CacheEntry>,
}

impl SkeletonCache {
    /// Load cache from disk, or return empty if missing.
    pub fn load() -> Self {
        let content = match fs::read_to_string(CACHE_FILE) {
            Ok(c) => c,
            Err(_) => return Self { files: HashMap::new() },
        };
        serde_json::from_str(&content).unwrap_or(Self { files: HashMap::new() })
    }

    /// Compute SHA256 hex digest of a file.
    fn file_hash(path: &Path) -> Result<String> {
        let mut file = fs::File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let hash = Sha256::digest(&buf);
        Ok(format!("{hash:x}"))
    }

    /// Get a cached skeleton if the file hasn't changed.
    /// Returns `None` if the file is new or modified.
    pub fn get(&self, path: &Path) -> Result<Option<FileSkeleton>> {
        let current_hash = Self::file_hash(path)?;
        let path_str = path.to_string_lossy().to_string();

        Ok(self.files.get(&path_str).and_then(|entry| {
            if entry.hash == current_hash {
                Some(entry.skeleton.clone())
            } else {
                None
            }
        }))
    }

    /// Insert/update a skeleton for a file path.
    pub fn set(&mut self, path: &Path, skeleton: &FileSkeleton) -> Result<()> {
        let hash = Self::file_hash(path)?;
        let path_str = path.to_string_lossy().to_string();

        self.files.insert(
            path_str,
            CacheEntry {
                hash,
                skeleton: skeleton.clone(),
            },
        );
        Ok(())
    }

    /// Get from cache or parse and cache.
    pub fn get_or_extract(&mut self, path: &Path) -> Result<FileSkeleton> {
        if let Some(skel) = self.get(path)? {
            return Ok(skel);
        }
        let skel = extract_skeleton(path)?;
        self.set(path, &skel)?;
        Ok(skel)
    }

    /// Save cache to disk.
    pub fn save(&self) -> Result<()> {
        let dir = Path::new(CACHE_DIR);
        if !dir.exists() {
            fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create {CACHE_DIR} directory"))?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(CACHE_FILE, &json).context("Failed to write cache file")?;
        Ok(())
    }
}
