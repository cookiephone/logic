use std::{fmt, rc::Rc};

use crate::{
    rewrite::{RewriteRule, RewriteRuleset},
    sat::DPLLSolver,
};

pub type Ident = u32;
pub type AST = Rc<ASTNode>;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEndOfInput,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    Variable(Ident),
    Not(AST),
    And(AST, AST),
    Or(AST, AST),
}

impl fmt::Display for ASTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Variable(identifier) => write!(f, "var{}", identifier),
            ASTNode::Not(p) => write!(f, "¬{}", p),
            ASTNode::And(lhs, rhs) => write!(f, "({} ∧ {})", lhs, rhs),
            ASTNode::Or(lhs, rhs) => write!(f, "({} ∨ {})", lhs, rhs),
        }
    }
}

pub trait AbstractSyntaxTree {
    fn variable(ident: u32) -> AST;
    fn and(&self, other: AST) -> AST;
    fn or(&self, other: AST) -> AST;
    fn not(&self) -> AST;
    fn dnf(&self) -> AST;
    fn cnf(&self) -> AST;
    fn sat(&self) -> bool;
}

impl AbstractSyntaxTree for AST {
    fn variable(ident: u32) -> AST {
        Rc::new(ASTNode::Variable(ident))
    }

    fn not(&self) -> AST {
        Rc::new(ASTNode::Not(self.clone()))
    }

    fn and(&self, other: AST) -> AST {
        Rc::new(ASTNode::And(self.clone(), other))
    }

    fn or(&self, other: AST) -> AST {
        Rc::new(ASTNode::Or(self.clone(), other))
    }

    fn dnf(&self) -> AST {
        let ruleset = RewriteRuleset {
            name: "DNF conversion",
            rules: vec![
                RewriteRule {
                    name: "double negation elimination",
                    top: procmacro::propositional_logic! { NOT NOT x },
                    bot: procmacro::propositional_logic! { x },
                },
                RewriteRule {
                    name: "de morgan's theorem for disjunction",
                    top: procmacro::propositional_logic! { NOT (x OR y) },
                    bot: procmacro::propositional_logic! { (NOT x AND NOT y) },
                },
                RewriteRule {
                    name: "de morgan's theorem for conjunction",
                    top: procmacro::propositional_logic! { NOT (x AND y) },
                    bot: procmacro::propositional_logic! { (NOT x OR NOT y) },
                },
                RewriteRule {
                    name: "left-distributive property of conjunction over disjunction",
                    top: procmacro::propositional_logic! { (x AND (y OR z)) },
                    bot: procmacro::propositional_logic! { ((x AND y) OR (x AND z)) },
                },
            ],
        };
        ruleset.rewrite_recursive_hull(self.clone())
    }

    fn cnf(&self) -> AST {
        let ruleset = RewriteRuleset {
            name: "CNF conversion",
            rules: vec![
                RewriteRule {
                    name: "double negation elimination",
                    top: procmacro::propositional_logic! { NOT NOT x },
                    bot: procmacro::propositional_logic! { x },
                },
                RewriteRule {
                    name: "de morgan's theorem for disjunction",
                    top: procmacro::propositional_logic! { NOT (x OR y) },
                    bot: procmacro::propositional_logic! { (NOT x AND NOT y) },
                },
                RewriteRule {
                    name: "de morgan's theorem for disjunction",
                    top: procmacro::propositional_logic! { NOT (x OR y) },
                    bot: procmacro::propositional_logic! { (NOT x AND NOT y) },
                },
                RewriteRule {
                    name: "left-distributive property of disjunction over conjunction",
                    top: procmacro::propositional_logic! { (x OR (y AND z)) },
                    bot: procmacro::propositional_logic! { ((x OR y) AND (x OR z)) },
                },
            ],
        };
        ruleset.rewrite_recursive_hull(self.clone())
    }

    fn sat(&self) -> bool {
        DPLLSolver::from(self).dpll()
    }
}
