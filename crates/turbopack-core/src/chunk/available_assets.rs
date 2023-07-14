use std::iter::once;

use anyhow::Result;
use indexmap::IndexSet;
use turbo_tasks::{
    graph::{AdjacencyMap, GraphTraversal},
    primitives::{BoolVc, U64Vc},
    TryJoinIterExt, ValueToString,
};
use turbo_tasks_hash::Xxh3Hash64Hasher;

use super::{ChunkableModuleReference, ChunkableModuleReferenceVc, ChunkingType};
use crate::{
    asset::Asset,
    module::{Module, ModuleVc, ModulesSetVc},
    reference::AssetReference,
};

/// Allows to gather information about which assets are already available.
/// Adding more roots will form a linked list like structure to allow caching
/// `include` queries.
#[turbo_tasks::value]
pub struct AvailableAssets {
    parent: Option<AvailableAssetsVc>,
    roots: Vec<ModuleVc>,
}

#[turbo_tasks::value_impl]
impl AvailableAssetsVc {
    #[turbo_tasks::function]
    fn new_normalized(parent: Option<AvailableAssetsVc>, roots: Vec<ModuleVc>) -> Self {
        AvailableAssets { parent, roots }.cell()
    }

    #[turbo_tasks::function]
    pub fn new(roots: Vec<ModuleVc>) -> Self {
        Self::new_normalized(None, roots)
    }

    #[turbo_tasks::function]
    pub async fn with_roots(self, roots: Vec<ModuleVc>) -> Result<Self> {
        let roots = roots
            .into_iter()
            .map(|root| async move { Ok((self.includes(root).await?, root)) })
            .try_join()
            .await?
            .into_iter()
            .filter_map(|(included, root)| (!*included).then_some(root))
            .collect();
        Ok(Self::new_normalized(Some(self), roots))
    }

    #[turbo_tasks::function]
    pub async fn hash(self) -> Result<U64Vc> {
        let this = self.await?;
        let mut hasher = Xxh3Hash64Hasher::new();
        if let Some(parent) = this.parent {
            hasher.write_value(parent.hash().await?);
        } else {
            hasher.write_value(0u64);
        }
        for root in &this.roots {
            hasher.write_value(root.ident().to_string().await?);
        }
        Ok(U64Vc::cell(hasher.finish()))
    }

    #[turbo_tasks::function]
    pub async fn includes(self, asset: ModuleVc) -> Result<BoolVc> {
        let this = self.await?;
        if let Some(parent) = this.parent {
            if *parent.includes(asset).await? {
                return Ok(BoolVc::cell(true));
            }
        }
        for root in this.roots.iter() {
            if chunkable_assets_set(*root).await?.contains(&asset) {
                return Ok(BoolVc::cell(true));
            }
        }
        Ok(BoolVc::cell(false))
    }
}

#[turbo_tasks::function]
async fn chunkable_assets_set(root: ModuleVc) -> Result<ModulesSetVc> {
    let assets =
        AdjacencyMap::new()
            .skip_duplicates()
            .visit(once(root), |&asset: &ModuleVc| async move {
                Ok(asset
                    .references()
                    .await?
                    .iter()
                    .copied()
                    .map(|reference| async move {
                        if let Some(chunkable) =
                            ChunkableModuleReferenceVc::resolve_from(reference).await?
                        {
                            if matches!(
                                &*chunkable.chunking_type().await?,
                                Some(
                                    ChunkingType::Parallel
                                        | ChunkingType::PlacedOrParallel
                                        | ChunkingType::Placed
                                )
                            ) {
                                return Ok(chunkable
                                    .resolve_reference()
                                    .primary_assets()
                                    .await?
                                    .iter()
                                    .map(|&asset| async move {
                                        Ok(ModuleVc::resolve_from(asset).await?)
                                    })
                                    .try_join()
                                    .await?
                                    .into_iter()
                                    .flatten()
                                    .collect());
                            }
                        }
                        Ok(Vec::new())
                    })
                    .try_join()
                    .await?
                    .into_iter()
                    .flatten()
                    .collect::<IndexSet<_>>())
            })
            .await
            .completed()?;
    Ok(ModulesSetVc::cell(
        assets.into_inner().into_reverse_topological().collect(),
    ))
}
