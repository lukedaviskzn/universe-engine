use std::{
    fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
};

use flate2::bufread::GzDecoder;
use serde::de::DeserializeOwned;

pub const MOD_DIR: &'static str = "data/mods";
pub const LOAD_ORDER: &'static str = "load_order.txt";
pub const MOD_META: &'static str = "mod.ron";
// pub const CORE_MOD: &'static str = "core";

#[derive(Debug, serde::Deserialize)]
pub struct ModMeta {
    pub name: String,
    pub version: semver::Version,
    pub engine_version: semver::VersionReq,
    pub author: String,
}

pub struct ModFs {
    mods: Vec<(ModMeta, PathBuf)>,
}

#[derive(Debug, thiserror::Error)]
pub enum ModError {
    #[error("io error: {0}")]
    IoError(#[from] io::Error),
    #[error("parse error: {0}")]
    RonParseError(#[from] ron::error::SpannedError),
    #[error("parse error: {0}")]
    BinParseError(#[from] bincode::Error),
    #[error("load order does any mods")]
    Empty,
}

impl ModFs {
    pub fn new() -> Result<ModFs, ModError> {
        let mod_dir = Path::new(MOD_DIR);
        
        let load_order = io::BufReader::new(fs::File::open(mod_dir.join(LOAD_ORDER))?);
        let load_order = load_order.lines()
            .map(|l| l.expect("failed to read load order file"))
            .filter(|l| l.len() > 0 && l.chars().all(|c| c.is_alphanumeric() || c == '_'))
            .collect::<Vec<_>>();

        if load_order.len() == 0 {
            return Err(ModError::Empty);
        }

        let mut mods = vec![];

        let engine_version = semver::Version::parse(std::env!("CARGO_PKG_VERSION")).expect("failed to get CARGO_PKG_VERSION environment variable");

        for m in load_order {
            let path = mod_dir.join(m);
            let meta: ModMeta = ron::from_str(&fs::read_to_string(path.join(MOD_META))?)?;

            if !meta.engine_version.matches(&engine_version) {
                log::error!("mod {:?} is not compatible with engine version {engine_version} (expected {})", meta.name, meta.engine_version);
                continue;
            }

            mods.push((meta, path));
        }

        Ok(ModFs { mods })
    }

    pub fn read_dir(&self, path: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
        let path = path.as_ref();
        let mut dir_contents = Vec::new();

        log::trace!("reading mod dir {path:?}");
        
        for (_, mod_path) in &self.mods {
            let dir_path = mod_path.join(path);
            for entry in dir_path.read_dir()? {
                let entry = entry?;

                // already found in mod with higher priority
                if dir_contents.iter().any(|(_, f)| *f == entry.file_name()) {
                    continue;
                }

                dir_contents.push((path.join(entry.file_name()), entry.file_name()));
            }
        }
        
        Ok(dir_contents.into_iter().map(|(p, _)| p).collect())
    }

    pub fn decompress_bin<T: DeserializeOwned>(&self, file: impl AsRef<Path>) -> Result<T, ModError> {
        let file = file.as_ref();

        log::trace!("decompressing binary ({}) {file:?}", std::any::type_name::<T>());
        
        let (_, mod_path) = self.mods.iter().filter(|(_, p)| p.join(file).exists()).last().ok_or(io::Error::from(io::ErrorKind::NotFound))?;
        let file = fs::File::open(mod_path.join(file))?;
        let reader = GzDecoder::new(io::BufReader::new(file));
        
        Ok(bincode::deserialize_from(reader)?)
    }
}
