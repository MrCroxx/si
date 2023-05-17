use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgError, Source};

use crate::{
    ActionFuncSpec, ActionFuncSpecKind,
    {node::PkgNode, spec::FuncUniqueId},
};

#[derive(Clone, Debug)]
pub struct SiPkgActionFunc<'a> {
    func_unique_id: FuncUniqueId,
    kind: ActionFuncSpecKind,
    hash: Hash,
    source: Source<'a>,
}

impl<'a> SiPkgActionFunc<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::ActionFunc(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::ACTION_FUNC_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        Ok(Self {
            func_unique_id: node.func_unique_id,
            kind: node.kind,
            hash: hashed_node.hash(),
            source: Source::new(graph, node_idx),
        })
    }

    pub fn func_unique_id(&self) -> FuncUniqueId {
        self.func_unique_id
    }

    pub fn kind(&self) -> ActionFuncSpecKind {
        self.kind
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
    }
}

impl<'a> TryFrom<SiPkgActionFunc<'a>> for ActionFuncSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgActionFunc<'a>) -> Result<Self, Self::Error> {
        Ok(ActionFuncSpec::builder()
            .kind(value.kind())
            .func_unique_id(value.func_unique_id)
            .build()?)
    }
}
