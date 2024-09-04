use serde::{Deserialize, Serialize};
use si_events::{ulid::Ulid, ContentHash};
use si_layer_cache::LayerDbError;
use thiserror::Error;

use super::{traits::SiVersionedNodeWeight, ContentNodeWeight, NodeWeightError};
use crate::{
    workspace_snapshot::graph::WorkspaceSnapshotGraphError, DalContext, WorkspaceSnapshotError,
    WorkspaceSnapshotGraphV3,
};

pub mod v1;

pub use v1::SchemaVariantNodeWeightV1;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SchemaVariantNodeWeightError {
    #[error("Invalid content for node weight: {0}")]
    InvalidContentForNodeWeight(Ulid),
    #[error("LayerDb error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("NodeWeight error: {0}")]
    NodeWeight(#[from] Box<NodeWeightError>),
    #[error("WorkspaceSnapshot error: {0}")]
    WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
    #[error("WorkspaceSnapshotGraph error: {0}")]
    WorkspaceSnapshotGraph(#[from] Box<WorkspaceSnapshotGraphError>),
}

pub type SchemaVariantNodeWeightResult<T> = Result<T, SchemaVariantNodeWeightError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SchemaVariantNodeWeight {
    V1(SchemaVariantNodeWeightV1),
}

impl SchemaVariantNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid, is_locked: bool, content_hash: ContentHash) -> Self {
        Self::V1(SchemaVariantNodeWeightV1::new(
            id,
            lineage_id,
            is_locked,
            content_hash,
        ))
    }

    pub async fn try_upgrade_from_content_node_weight(
        ctx: &DalContext,
        v3_graph: &mut WorkspaceSnapshotGraphV3,
        content_node_weight: &ContentNodeWeight,
    ) -> SchemaVariantNodeWeightResult<()> {
        SchemaVariantNodeWeightV1::try_upgrade_from_content_node_weight(
            ctx,
            v3_graph,
            content_node_weight,
        )
        .await
    }
}

impl SiVersionedNodeWeight for SchemaVariantNodeWeight {
    type Inner = SchemaVariantNodeWeightV1;

    /// Return a reference to the most uup to date enum variant
    fn inner(&self) -> &SchemaVariantNodeWeightV1 {
        match self {
            SchemaVariantNodeWeight::V1(inner) => inner,
        }
    }

    /// Return a mutable reference to the most up to date enum variant
    fn inner_mut(&mut self) -> &mut SchemaVariantNodeWeightV1 {
        match self {
            SchemaVariantNodeWeight::V1(inner) => inner,
        }
    }
}
