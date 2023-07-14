use anyhow::{bail, Result};
use indexmap::IndexSet;

use crate::{
    asset::{Asset, AssetVc},
    ident::AssetIdentVc,
    raw_module::RawModuleVc,
    source::SourceVc,
};

/// A module. This usually represents parsed source code, which has references
/// to other modules.
#[turbo_tasks::value_trait]
pub trait Module: Asset {
    /// The identifier of the [Module]. It's expected to be unique and capture
    /// all properties of the [Module].
    fn ident(&self) -> AssetIdentVc;
}

#[turbo_tasks::value(transparent)]
pub struct OptionModule(Option<ModuleVc>);

#[turbo_tasks::value(transparent)]
pub struct Modules(Vec<ModuleVc>);

#[turbo_tasks::value_impl]
impl ModulesVc {
    #[turbo_tasks::function]
    pub fn empty() -> Self {
        Self::cell(Vec::new())
    }
}

/// A set of [Module]s
#[turbo_tasks::value(transparent)]
pub struct ModulesSet(IndexSet<ModuleVc>);

/// This is a temporary function that should be removed once the [Module]
/// trait completely replaces the [Asset] trait.
/// It converts an [Asset] into a [Module], but either casting it or wrapping it
/// in a [RawModule].
// TODO make this function unnecessary, it should never be a Source
#[turbo_tasks::function]
pub async fn convert_asset_to_module(asset: AssetVc) -> Result<ModuleVc> {
    if let Some(module) = ModuleVc::resolve_from(asset).await? {
        Ok(module)
    } else if let Some(source) = SourceVc::resolve_from(asset).await? {
        Ok(RawModuleVc::new(source).into())
    } else {
        bail!("Asset must be a Module or a Source")
    }
}
