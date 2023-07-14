use crate::{
    asset::{Asset, AssetVc},
    module::{Module, ModuleVc},
};

/// An [Asset] that should never be placed into a chunk, but whose references
/// should still be followed.
#[turbo_tasks::value_trait]
pub trait PassthroughAsset: Module + Asset {}
