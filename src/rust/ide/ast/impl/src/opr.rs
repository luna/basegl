//! Utilities for dealing with operators and Ast nodes related to them, like `Infix`, `Section*`.

use crate::prelude::*;

use crate::Ast;
use crate::assoc::Assoc;
use crate::known;
use crate::Shape;
use crate::crumbs::{Crumb, Located, InfixCrumb, SectionLeftCrumb, SectionRightCrumb, SectionSidesCrumb};

/// Identifiers of operators with special meaning for IDE.
pub mod predefined {
    /// Used to create bindings, e.g. `add a b = a + b` or `foo = 5`.
    pub const ASSIGNMENT : &str = "=";
    /// Used to create type paths (like `Int.+` or `IO.println`).
    pub const ACCESS : &str = ".";
}

/// Checks if given Ast is an assignment operator identifier.
pub fn is_assignment_opr(ast:&Ast) -> bool {
    let opr_opt = known::Opr::try_from(ast);
    opr_opt.map(|opr| opr.name == predefined::ASSIGNMENT).unwrap_or(false)
}

/// If given Ast is an assignment operator, returns it as Some known::Infix.
pub fn to_assignment(ast:&Ast) -> Option<known::Infix> {
    let infix = known::Infix::try_from(ast).ok()?;
    is_assignment_opr(&infix.opr).then(infix)
}

/// Checks if a given node is an assignment infix expression.
pub fn is_assignment(ast:&Ast) -> bool {
    let infix = known::Infix::try_from(ast);
    infix.map(|infix| is_assignment_opr(&infix.opr)).unwrap_or(false)
}

/// Infix operator operand. Optional, as we deal with Section* nodes as well.
pub type Operand = Option<Located<Ast>>;

/// Infix operator standing between (optional) operands.
pub type Operator = Located<known::Opr>;

fn make_operand(parent:&Located<Ast>, crumb:impl Into<Crumb>, child:&Ast) -> Operand {
    Some(parent.child(crumb,child.clone()))
}

fn make_operator(parent:&Located<Ast>, crumb:impl Into<Crumb>, opr:&Ast) -> Option<Operator> {
    let opr = known::Opr::try_from(opr).ok()?;
    Some(parent.child(crumb,opr))
}

fn assoc(ast:&known::Opr) -> Assoc {
    Assoc::of(&ast.name)
}


// ========================
// === GeneralizedInfix ===
// ========================

/// An abstraction over `Infix` and all `SectionSth` nodes.
#[derive(Clone,Debug)]
pub struct GeneralizedInfix {
    /// Left operand, if present.
    pub left  : Operand,
    /// The operator, always present.
    pub opr   : Operator,
    /// Right operand, if present.
    pub right : Operand,
}

impl GeneralizedInfix {
    pub fn try_new_root(ast:&Ast) -> Option<GeneralizedInfix> {
        GeneralizedInfix::try_new(&Located::new_root(ast.clone()))
    }

    /// Tries interpret given AST node as GeneralizedInfix. Returns None, if Ast is not any kind of
    /// application on infix operator.
    pub fn try_new(ast:&Located<Ast>) -> Option<GeneralizedInfix> {
        match ast.shape() {
            Shape::Infix(infix) => Some(GeneralizedInfix{
                left  : make_operand (ast,InfixCrumb::LeftOperand, &infix.larg),
                opr   : make_operator(ast,InfixCrumb::Operator,    &infix.opr)?,
                right : make_operand (ast,InfixCrumb::RightOperand,&infix.rarg),
            }),
            Shape::SectionLeft(left) => Some(GeneralizedInfix{
                left  : make_operand (ast,SectionLeftCrumb::Arg,&left.arg),
                opr   : make_operator(ast,SectionLeftCrumb::Opr,&left.opr)?,
                right : None,
            }),
            Shape::SectionRight(right) => Some(GeneralizedInfix{
                left  : None,
                opr   : make_operator(ast,SectionRightCrumb::Opr,&right.opr)?,
                right : make_operand (ast,SectionRightCrumb::Arg,&right.arg),
            }),
            Shape::SectionSides(sides) => Some(GeneralizedInfix{
                left  : None,
                opr   : make_operator(ast,SectionSidesCrumb,&sides.opr)?,
                right : None,
            }),
            _ => None,
        }
    }

    /// Associativity of the operator used in this infix expression.
    pub fn assoc(&self) -> Assoc {
        Assoc::of(&self.name())
    }

    /// Identifier name  of the operator used in this infix expression.
    pub fn name(&self) -> &str {
        &self.opr.name
    }

    /// The self operand, target of the application.
    pub fn target_operand(&self) -> Operand {
        match self.assoc() {
            Assoc::Left  => self.left.clone(),
            Assoc::Right => self.right.clone(),
        }
    }

    /// Operand other than self.
    pub fn argument_operand(&self) -> Operand {
        match self.assoc() {
            Assoc::Left  => self.right.clone(),
            Assoc::Right => self.left.clone(),
        }
    }

    /// Converts chain of infix applications using the same operator into `Chain`.
    /// Sample inputs are `x,y,x` or `a+b+` or `+5+5+5`. Note that `Sides*` nodes
    /// are also supported, along the `Infix` nodes.
    pub fn flatten(&self) -> Chain {
        let target = self.target_operand();
        let rest   = ChainElement {
            operator : self.opr.clone(),
            operand  : self.argument_operand()
        };

        let target_subtree_infix = target.clone().and_then(|ast| {
            GeneralizedInfix::try_new(&ast)
        });
        let mut target_subtree_flat = match target_subtree_infix {
            Some(target_infix) if target_infix.name() == self.name() =>
                target_infix.flatten(),
            _ => Chain { target, args:Vec::new(), operator:self.opr.item.clone() },
        };

        target_subtree_flat.args.push(rest);
        target_subtree_flat
    }
}



// =============
// === Chain ===
// =============

/// Result of flattening infix operator chain, like `a+b+c` or `Foo.Bar.Baz`.
#[derive(Clone,Debug)]
pub struct Chain {
    /// The primary application target (left- or right-most operand, depending on
    /// operators associativity).
    pub target : Operand,
    /// Subsequent operands applied to the `target`.
    pub args   : Vec<ChainElement>,
    /// Operator.
    pub operator : known::Opr,
}

impl Chain {
    /// If this is infix, it flattens whole chain and returns result.
    /// Otherwise, returns None.
    pub fn try_new(ast:&Ast) -> Option<Chain> {
        GeneralizedInfix::try_new_root(&ast).map(|infix| infix.flatten())
    }

    /// Flattens infix chain if this is infix application of given operator.
    pub fn try_new_of(ast:&Ast, operator:&str) -> Option<Chain> {
        let infix = GeneralizedInfix::try_new_root(&ast)?;
        (infix.name() == operator).as_some_from(|| infix.flatten())
    }

    /// Iterates over &Located<Ast>, beginning with target (this argument) and then subsequent
    /// arguments.
    pub fn enumerate_operands<'a>(&'a self) -> impl Iterator<Item=&'a Located<Ast>> + 'a {
        let this = std::iter::once(&self.target);
        let args = self.args.iter().map(|elem| &elem.operand);
        let operands = this.chain(args).flatten();
        operands
    }

    /// Iterates over &Located<Ast>, beginning with target (this argument) and then subsequent
    /// arguments.
    pub fn enumerate_operators<'a>(&'a self) -> impl Iterator<Item=&'a Located<known::Opr>> + 'a {
        self.args.iter().map(|elem| &elem.operator)
    }
}

/// Element of the infix application chain, i.e. operator and its operand.
#[derive(Clone,Debug)]
pub struct ChainElement {
    #[allow(missing_docs)]
    pub operator : Operator,
    /// Operand on the opposite side to `this` argument.
    /// Depending on operator's associativity it is either right (for left-associative operators)
    /// or on the left side of operator.
    pub operand  : Operand,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infix_chain_tests() {
        let a = Ast::var("a");
        let b = Ast::var("b");
        let c = Ast::var("c");
        let a_plus_b = Ast::infix(a.clone(),"+",b.clone());
        let a_plus_b_plus_c = Ast::infix(a_plus_b.clone(),"+",c.clone());


        let chain = Chain::try_new(&a_plus_b_plus_c).unwrap();

        let expect_ast_at_crumb_for = |operand:&Operand, expected_ast:&Ast| {
            let crumbs = &operand.as_ref().unwrap().crumbs;
            let ast    = a_plus_b_plus_c.get_traversing(crumbs).unwrap();
            assert_eq!(ast, expected_ast, "expected `{}` at crumbs `{:?}` for `{}`",
                       expected_ast.repr(), crumbs, a_plus_b_plus_c.repr());
        };

        assert_eq!(chain.target.as_ref().unwrap().item, a);
        assert_eq!(chain.args[0].operand.as_ref().unwrap().item, b);
        assert_eq!(chain.args[1].operand.as_ref().unwrap().item, c);
        expect_ast_at_crumb_for(&chain.target, &a);
        expect_ast_at_crumb_for(&chain.args[0].operand, &b);
        expect_ast_at_crumb_for(&chain.args[1].operand, &c);
    }
}