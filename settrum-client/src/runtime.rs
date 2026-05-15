//! Subxt-generated bindings for the Settrum runtime.
//!
//! The macro reads `artifacts/metadata.scale` (committed to the repo) and
//! produces typed APIs for every pallet, storage item, extrinsic and event.
//! Regenerate the metadata file when the runtime changes — see the crate
//! root for the command.

#[subxt::subxt(runtime_metadata_path = "artifacts/metadata.scale")]
pub mod api {}
