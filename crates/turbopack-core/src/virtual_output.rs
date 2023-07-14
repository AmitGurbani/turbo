use anyhow::Result;
use turbo_tasks_fs::FileSystemPathVc;

use crate::{
    asset::{Asset, AssetContentVc, AssetVc},
    ident::AssetIdentVc,
    output::{OutputAsset, OutputAssetVc},
};

/// An [Asset] that is created from some passed source code.
#[turbo_tasks::value]
pub struct VirtualOutputAsset {
    pub path: FileSystemPathVc,
    pub content: AssetContentVc,
}

#[turbo_tasks::value_impl]
impl VirtualOutputAssetVc {
    #[turbo_tasks::function]
    pub fn new(path: FileSystemPathVc, content: AssetContentVc) -> Self {
        Self::cell(VirtualOutputAsset { path, content })
    }
}

#[turbo_tasks::value_impl]
impl OutputAsset for VirtualOutputAsset {
    #[turbo_tasks::function]
    fn ident(&self) -> AssetIdentVc {
        AssetIdentVc::from_path(self.path)
    }
}

#[turbo_tasks::value_impl]
impl Asset for VirtualOutputAsset {
    #[turbo_tasks::function]
    fn content(&self) -> AssetContentVc {
        self.content
    }
}
