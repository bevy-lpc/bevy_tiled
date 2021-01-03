use std::path::PathBuf;

use crate::map::Map;
use anyhow::Result;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    utils::BoxedFuture,
};
use tiled::{TiledFSHandler, TiledIOHandler, TiledParser};



struct WrappedIO {
    handler: TiledFSHandler,
    asset_path: String
}

impl TiledIOHandler for WrappedIO {
    
    fn read_bytes<'a>(&self, path: &'a std::path::Path) -> Result<Vec<u8>, std::io::Error> {
        let target = PathBuf::from(self.asset_path.clone()).join(path);
        self.handler.read_bytes(&target)
    }

    fn resolve_path<'a, 'b>(&self, base: &'a std::path::Path, path: &'a std::path::Path) -> std::path::PathBuf {
        self.handler.resolve_path(base, path)
    }
}
impl Default for WrappedIO {
    fn default() -> Self {
        Self{
            asset_path: "assets".to_string(),
            handler: TiledFSHandler{},
        }
    }
}


const FLIPPED_HORIZONTALLY_FLAG: u32 = 0x80000000;
const FLIPPED_VERTICALLY_FLAG: u32 = 0x40000000;
const FLIPPED_DIAGONALLY_FLAG: u32 = 0x20000000;
const ALL_FLIP_FLAGS: u32 =
    FLIPPED_HORIZONTALLY_FLAG | FLIPPED_VERTICALLY_FLAG | FLIPPED_DIAGONALLY_FLAG;

pub struct TiledMapLoader{
    tiled: TiledParser,
}

impl Default for TiledMapLoader {
    fn default() -> Self {
        let boxed : std::sync::Arc<dyn TiledIOHandler + Sync  + Send> = std::sync::Arc::new(WrappedIO::default());
        Self {
            tiled: TiledParser::new(boxed)
        }
    }
}

impl TiledMapLoader {
    pub fn remove_tile_flags(tile: u32) -> u32 {
        let tile = tile & !ALL_FLIP_FLAGS;
        tile
    }
}

impl AssetLoader for TiledMapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let path = load_context.path();
            println!("{:?}", path);
            let tiled_map = self.tiled.parse_map(bytes, Some(path))?;

            let map = Map::try_from(path, tiled_map)?;
            load_context.set_default_asset(LoadedAsset::new(map));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}
