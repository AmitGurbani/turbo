use anyhow::Result;
use serde_json::json;
use turbo_tasks_fs::{File, FileSystem};
use turbopack_core::{
    asset::{Asset, AssetContentVc, AssetVc},
    ident::AssetIdentVc,
    output::{OutputAsset, OutputAssetVc},
    reference::all_assets,
};

#[turbo_tasks::value(shared)]
pub struct NftJsonAsset {
    entry: AssetVc,
}

#[turbo_tasks::value_impl]
impl NftJsonAssetVc {
    #[turbo_tasks::function]
    pub fn new(entry: AssetVc) -> Self {
        Self::cell(NftJsonAsset { entry })
    }
}

#[turbo_tasks::value_impl]
impl OutputAsset for NftJsonAsset {}

#[turbo_tasks::value_impl]
impl Asset for NftJsonAsset {
    #[turbo_tasks::function]
    async fn ident(&self) -> Result<AssetIdentVc> {
        let path = self.entry.ident().path().await?;
        Ok(AssetIdentVc::from_path(
            path.fs.root().join(&format!("{}.nft.json", path.path)),
        ))
    }

    #[turbo_tasks::function]
    async fn content(&self) -> Result<AssetContentVc> {
        let context = self.entry.ident().path().parent().await?;
        // For clippy -- This explicit deref is necessary
        let entry_path = &*self.entry.ident().path().await?;
        let mut result = Vec::new();
        if let Some(self_path) = context.get_relative_path_to(entry_path) {
            let set = all_assets(self.entry);
            for asset in set.await?.iter() {
                let path = asset.ident().path().await?;
                if let Some(rel_path) = context.get_relative_path_to(&path) {
                    if rel_path != self_path {
                        result.push(rel_path);
                    }
                }
            }
            result.sort();
            result.dedup();
        }
        let json = json!({
          "version": 1,
          "files": result
        });

        Ok(File::from(json.to_string()).into())
    }
}
