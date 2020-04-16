//! An utility builder to be used in tests.
use crate::node;
use crate::Node;
use crate::SpanTree;

use data::text::Size;



// =====================
// === Builder Trait ===
// =====================

/// A trait with common operations for all builders.
pub trait Builder : Sized {
    /// Reference to currently built  node.
    fn built_node(&mut self) -> &mut Node;

    /// Add new AST-type child to node. Returns the child's builder which may be used to further
    /// extending this branch of the tree.
    fn add_ast_child<Cbs>(self, offset:usize, len:usize, crumbs:Cbs) -> ChildBuilder<Self>
        where Cbs : IntoIterator<Item:Into<ast::crumbs::Crumb>> {
        let node = Node {
            node_type : node::Type::Ast,
            len       : Size::new(len),
            children  : vec![]
        };
        let child = node::Child { node,
            offset              : Size::new(offset),
            chained_with_parent : false,
            ast_crumbs          : crumbs.into_iter().map(|cb| cb.into()).collect(),
        };
        ChildBuilder {
            built  : child,
            parent : self
        }
    }

    /// Add a leaf AST-type child to node.
    fn add_ast_leaf<Cbs>(self, offset:usize, len:usize, crumbs:Cbs) -> Self
        where Cbs : IntoIterator<Item:Into<ast::crumbs::Crumb>> {
        self.add_ast_child(offset,len,crumbs).done()
    }

    /// Add an Empty-type child to node.
    fn add_empty_child(mut self, offset:usize) -> Self {
        let node = Node::new_empty();
        let child = node::Child { node,
            offset : Size::new(offset),
            chained_with_parent : false,
            ast_crumbs          : vec![]
        };
        self.built_node().children.push(child);
        self
    }
}



/// ================
/// === Builders ===
/// ================

// === SpanTree Builder ===

/// The main builder for SpanTree.
#[derive(Debug)]
pub struct TreeBuilder {
    built : Node,
}

impl TreeBuilder {
    /// Create new builder for tree with root having length `len`.
    pub fn new(len:usize) -> Self {
        TreeBuilder {
            built : Node {
                node_type : node::Type::Ast,
                len       : Size::new(len),
                children  : vec![],
            }
        }
    }

    /// Return the built SpanTree.
    pub fn build(self) -> SpanTree {
        SpanTree {
            root : self.built
        }
    }
}

impl Builder for TreeBuilder {
    fn built_node(&mut self) -> &mut Node {
        &mut self.built
    }
}


// === Child Node Builder ===

/// A builder for some child node. This builder may be returned from `add_ast_child` function.
#[derive(Debug)]
pub struct ChildBuilder<Parent> {
    built  : node::Child,
    parent : Parent,
}

impl<Parent:Builder> ChildBuilder<Parent> {

    /// Set the child as being chained with parent.
    pub fn chain_with_parent(mut self) -> Self {
        self.built.chained_with_parent = true;
        self
    }

    /// Finish child building and return builder of the node's Parent.
    pub fn done(mut self) -> Parent {
        self.parent.built_node().children.push(self.built);
        self.parent
    }
}

impl<T> Builder for ChildBuilder<T> {
    fn built_node(&mut self) -> &mut Node {
        &mut self.built.node
    }
}
