use core::ast::{AbstractSyntaxTree, AST};

fn main() {
    let formula = procmacro::propositional_logic! { NOT NOT (((NOT (c OR d) AND (a OR NOT NOT b)) AND NOT a) AND NOT b) };
    println!("formula:     {}", formula);
    println!("dnf:         {}", formula.dnf());
    println!("cnf:         {}", formula.cnf());
    println!("satisfiable: {}", formula.sat());
}
