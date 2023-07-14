use std::fmt::Debug;

use anyhow::Result;
use turbo_tasks::primitives::{BoolVc, StringVc};
use turbo_tasks_fs::FileSystemPathVc;

use super::{ChunkVc, EvaluatableAssetsVc};
use crate::{
    environment::EnvironmentVc,
    ident::AssetIdentVc,
    module::ModuleVc,
    output::{OutputAssetVc, OutputAssetsVc},
};

/// A context for the chunking that influences the way chunks are created
#[turbo_tasks::value_trait]
pub trait ChunkingContext {
    fn context_path(&self) -> FileSystemPathVc;
    fn output_root(&self) -> FileSystemPathVc;

    // TODO remove this, a chunking context should not be bound to a specific
    // environment since this can change due to transitions in the module graph
    fn environment(&self) -> EnvironmentVc;

    // TODO(alexkirsz) Remove this from the chunking context. This should be at the
    // discretion of chunking context implementors. However, we currently use this
    // in a couple of places in `turbopack-css`, so we need to remove that
    // dependency first.
    fn chunk_path(&self, ident: AssetIdentVc, extension: &str) -> FileSystemPathVc;

    // TODO(alexkirsz) Remove this from the chunking context.
    /// Reference Source Map Assets for chunks
    fn reference_chunk_source_maps(&self, chunk: OutputAssetVc) -> BoolVc;

    fn can_be_in_same_chunk(&self, asset_a: ModuleVc, asset_b: ModuleVc) -> BoolVc;

    fn asset_path(
        &self,
        content_hash: &str,
        original_asset_ident: AssetIdentVc,
    ) -> FileSystemPathVc;

    fn is_hot_module_replacement_enabled(&self) -> BoolVc {
        BoolVc::cell(false)
    }

    fn layer(&self) -> StringVc {
        StringVc::cell("".to_string())
    }

    fn with_layer(&self, layer: &str) -> ChunkingContextVc;

    fn chunk_group(&self, entry: ChunkVc) -> OutputAssetsVc;

    fn evaluated_chunk_group(
        &self,
        entry: ChunkVc,
        evaluatable_assets: EvaluatableAssetsVc,
    ) -> OutputAssetsVc;
}
