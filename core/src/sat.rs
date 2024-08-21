use std::{
    collections::{HashMap, HashSet},
    fmt, vec,
};

use crate::ast::{ASTNode, AbstractSyntaxTree, Ident, AST};

#[derive(Hash, PartialEq, Eq, Clone)]
enum Polarity {
    Positive,
    Negative,
}

impl Polarity {
    fn flip(&self) -> Polarity {
        match self {
            Polarity::Positive => Polarity::Negative,
            Polarity::Negative => Polarity::Positive,
        }
    }
}

#[derive(Hash, PartialEq, Eq, Clone)]
struct Literal {
    identifier: Ident,
    polarity: Polarity,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.polarity {
            Polarity::Positive => write!(f, "var{}", self.identifier),
            Polarity::Negative => write!(f, "¬var{}", self.identifier),
        }
    }
}

impl Literal {
    fn not(&self) -> Self {
        Self {
            identifier: self.identifier,
            polarity: self.polarity.flip(),
        }
    }
}

#[derive(Clone)]
struct Clause {
    literals: HashSet<Literal>,
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{{}}}",
            self.literals
                .iter()
                .map(|lit| lit.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Clause {
    fn unit(unit: Literal) -> Self {
        Self {
            literals: HashSet::from([unit]),
        }
    }

    fn is_unit_clause(&self) -> bool {
        self.literals.len() == 1
    }

    fn is_empty(&self) -> bool {
        self.literals.is_empty()
    }

    fn contains(&self, literal: &Literal) -> bool {
        self.literals.contains(literal)
    }

    fn remove(&mut self, literal: &Literal) {
        self.literals.remove(literal);
    }
}

#[derive(Clone)]
pub struct DPLLSolver {
    clauses: Vec<Clause>,
}

impl fmt::Display for DPLLSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{{}}}",
            self.clauses
                .iter()
                .map(|clause| clause.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl From<&AST> for DPLLSolver {
    fn from(value: &AST) -> Self {
        Self {
            clauses: generate_clauses_from_tree(value.cnf()),
        }
    }
}

impl DPLLSolver {
    fn get_unit_clause(&self) -> Option<&Literal> {
        self.clauses
            .iter()
            .find(|clause| clause.is_unit_clause())
            .map(|clause| clause.literals.iter().next().unwrap())
    }

    fn unit_propagate(&mut self, unit: &Literal) {
        self.clauses.retain(|clause| !clause.contains(unit));
        let not_unit = unit.not();
        self.clauses
            .iter_mut()
            .for_each(|clause| clause.remove(&not_unit));
    }

    fn unit_propagation(&mut self) {
        while let Some(unit) = self.get_unit_clause().cloned() {
            self.unit_propagate(&unit);
        }
    }

    fn pure_literal_elimination(&mut self) {
        let mut purity_table = HashMap::new();
        for clause in &self.clauses {
            for literal in &clause.literals {
                match purity_table.get_mut(literal) {
                    Some((_, false)) => continue,
                    Some((polarity, purity)) => *purity = *polarity == literal.polarity,
                    None => {
                        purity_table.insert(literal.clone(), (literal.polarity.clone(), true));
                    }
                }
            }
        }
        purity_table.retain(|_, (_, purity)| *purity);
        for literal in purity_table.keys() {
            self.clauses.retain(|clause| clause.contains(literal));
        }
    }

    fn with_unit_clause(&mut self, unit: Literal) -> Self {
        let mut new = self.clone();
        new.clauses.push(Clause::unit(unit));
        new
    }

    fn choose_literal(&self) -> Literal {
        self.clauses[0].literals.iter().next().unwrap().clone()
    }

    pub fn dpll(&mut self) -> bool {
        self.unit_propagation();
        self.pure_literal_elimination();
        if self.clauses.is_empty() {
            return true;
        }
        if self.clauses.iter().any(|clause| clause.is_empty()) {
            return false;
        }
        let unit = self.choose_literal();
        self.with_unit_clause(unit.not()).dpll() || self.with_unit_clause(unit).dpll()
    }
}

fn generate_clauses_from_tree(ast: AST) -> Vec<Clause> {
    let mut clauses = Vec::new();
    let mut subtrees = vec![&ast];
    while let Some(subtree) = subtrees.pop() {
        match &**subtree {
            ASTNode::And(p1, p2) => {
                subtrees.push(p1);
                subtrees.push(p2);
            }
            _ => clauses.push(generate_clause_from_subtree(subtree)),
        }
    }
    clauses
}

fn generate_clause_from_subtree(ast: &AST) -> Clause {
    let mut literals = HashSet::new();
    let mut subtrees = vec![ast];
    while !subtrees.is_empty() {
        match &**subtrees.pop().unwrap() {
            ASTNode::Variable(ident) => {
                literals.insert(Literal {
                    identifier: *ident,
                    polarity: Polarity::Positive,
                });
            }
            ASTNode::Not(variable) => match &**variable {
                ASTNode::Variable(ident) => {
                    literals.insert(Literal {
                        identifier: *ident,
                        polarity: Polarity::Negative,
                    });
                }
                _ => unreachable!(),
            },
            ASTNode::Or(p1, p2) => {
                subtrees.push(p1);
                subtrees.push(p2);
            }
            _ => unreachable!(),
        }
    }
    Clause { literals }
}
