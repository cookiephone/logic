use std::{collections::HashMap, fmt, rc::Rc};

use crate::ast::{ASTNode, Ident, AST};

#[derive(Debug)]
pub enum RewriteError {
    RuleDoesNotApply,
}

pub struct RewriteRule {
    pub name: &'static str,
    pub top: AST,
    pub bot: AST,
}

impl fmt::Display for RewriteRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} ⊢ {} ({})", self.top, self.bot, self.name)
    }
}

impl RewriteRule {
    pub fn rewrite(&self, target: AST) -> AST {
        let matching = match Self::matching(&target, &self.top) {
            Ok(m) => m,
            Err(_) => return target,
        };
        Self::substitute(self.bot.clone(), &matching)
    }

    fn matching(target: &AST, pattern: &AST) -> Result<HashMap<Ident, AST>, RewriteError> {
        match (&**pattern, &**target) {
            (ASTNode::Not(template_p), ASTNode::Not(p)) => Ok(Self::matching(p, template_p)?),
            (ASTNode::And(template_p1, template_p2), ASTNode::And(p1, p2)) => {
                let mut matching_p1 = Self::matching(p1, template_p1)?;
                matching_p1.extend(Self::matching(p2, template_p2)?);
                Ok(matching_p1)
            }
            (ASTNode::Or(template_p1, template_p2), ASTNode::Or(p1, p2)) => {
                let mut matching_p1 = Self::matching(p1, template_p1)?;
                matching_p1.extend(Self::matching(p2, template_p2)?);
                Ok(matching_p1)
            }
            (ASTNode::Variable(template_ident), _) => {
                Ok(HashMap::from([(*template_ident, target.clone())]))
            }
            _ => Err(RewriteError::RuleDoesNotApply),
        }
    }

    fn substitute(template: AST, matching: &HashMap<Ident, AST>) -> AST {
        match &*template {
            ASTNode::Variable(ident) => matching.get(ident).unwrap().clone(),
            ASTNode::Not(p) => Rc::new(ASTNode::Not(Self::substitute(p.clone(), matching))),
            ASTNode::And(p1, p2) => Rc::new(ASTNode::And(
                Self::substitute(p1.clone(), matching),
                Self::substitute(p2.clone(), matching),
            )),
            ASTNode::Or(p1, p2) => Rc::new(ASTNode::Or(
                Self::substitute(p1.clone(), matching),
                Self::substitute(p2.clone(), matching),
            )),
        }
    }
}

pub struct RewriteRuleset {
    pub name: &'static str,
    pub rules: Vec<RewriteRule>,
}

impl fmt::Display for RewriteRuleset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.name)?;
        for rule in &self.rules {
            write!(f, "{}", rule)?;
        }
        Ok(())
    }
}

impl RewriteRuleset {
    pub fn rewrite(&self, target: AST) -> AST {
        self.rules
            .iter()
            .fold(target, |ast, rule| rule.rewrite(ast))
    }

    pub fn rewrite_recursive(&self, mut target: AST) -> AST {
        target = self.rewrite(target);
        match &*target {
            ASTNode::Not(p) => target = Rc::new(ASTNode::Not(self.rewrite_recursive(p.clone()))),
            ASTNode::And(p1, p2) => {
                target = Rc::new(ASTNode::And(
                    self.rewrite_recursive(p1.clone()),
                    self.rewrite_recursive(p2.clone()),
                ))
            }
            ASTNode::Or(p1, p2) => {
                target = Rc::new(ASTNode::Or(
                    self.rewrite_recursive(p1.clone()),
                    self.rewrite_recursive(p2.clone()),
                ))
            }
            _ => (),
        }
        target
    }

    pub fn rewrite_recursive_hull(&self, mut target: AST) -> AST {
        loop {
            let new = self.rewrite_recursive(target.clone());
            if new == target {
                return target;
            }
            target = new;
        }
    }
}
