use anyhow::{Context, Result};
use indexmap::IndexSet;

use crate::{
    asset::{Asset, AssetVc},
    ident::AssetIdentVc,
};

/// An asset that should be outputted, e. g. written to disk or served from a
/// server.
#[turbo_tasks::value_trait]
pub trait OutputAsset: Asset {
    // TODO change this to path() -> FileSystemPathVc
    /// The identifier of the [OutputAsset]. It's expected to be unique and
    /// capture all properties of the [OutputAsset]. Only path must be used.
    fn ident(&self) -> AssetIdentVc;
}

#[turbo_tasks::value(transparent)]
pub struct OutputAssets(Vec<OutputAssetVc>);

#[turbo_tasks::value_impl]
impl OutputAssetsVc {
    #[turbo_tasks::function]
    pub fn empty() -> Self {
        Self::cell(Vec::new())
    }
}

/// A set of [OutputAsset]s
#[turbo_tasks::value(transparent)]
pub struct OutputAssetsSet(IndexSet<OutputAssetVc>);

/// This is a temporary function that should be removed once the [OutputAsset]
/// trait completely replaces the [Asset] trait.
/// TODO make this function unnecessary
#[turbo_tasks::function]
pub async fn asset_to_output_asset(asset: AssetVc) -> Result<OutputAssetVc> {
    OutputAssetVc::resolve_from(asset)
        .await?
        .context("Asset must be a OutputAsset")
}
