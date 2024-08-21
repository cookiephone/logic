extern crate proc_macro;

use std::collections::HashMap;

use proc_macro::{TokenStream, TokenTree};

#[proc_macro]
pub fn propositional_logic(input: TokenStream) -> TokenStream {
    let mut tokens = vec![];
    tokenize_propositional_logic(input, &mut tokens);
    let mut ast_code = "".to_owned();
    let (mut symtab, mut n) = (HashMap::new(), 0);
    codegen(
        &mut tokens.into_iter(),
        &mut ast_code,
        (&mut symtab, &mut n),
    );
    ast_code.parse().unwrap()
}

fn codegen<I: Iterator<Item = String>>(
    tokens: &mut I,
    code: &mut String,
    (symtab, n): (&mut HashMap<String, i32>, &mut i32),
) {
    if let Some(token) = tokens.next() {
        match token.as_str() {
            "NOT" => {
                codegen(tokens, code, (symtab, n));
                code.push_str(".not()");
            }
            "AND" => {
                codegen(tokens, code, (symtab, n));
                code.push_str(".and(");
                codegen(tokens, code, (symtab, n));
                code.push(')');
            }
            "OR" => {
                codegen(tokens, code, (symtab, n));
                code.push_str(".or(");
                codegen(tokens, code, (symtab, n));
                code.push(')');
            }
            ident => {
                code.push_str(&format!(
                    "<AST as AbstractSyntaxTree>::variable({})",
                    match symtab.get(ident) {
                        Some(id) => *id,
                        None => {
                            symtab.insert(ident.to_owned(), *n);
                            *n += 1;
                            *n - 1
                        }
                    }
                ));
            }
        }
    }
}

fn tokenize_propositional_logic(stream: TokenStream, tokens: &mut Vec<String>) {
    for tree in stream {
        match tree {
            TokenTree::Group(group) => {
                let group_trees = group.stream().into_iter().collect::<Vec<_>>();
                let mut operator_index = 0;
                for subtree in &group_trees {
                    match subtree {
                        TokenTree::Group(_) => operator_index += 1,
                        TokenTree::Ident(ident)
                            if !matches!(ident.to_string().as_str(), "AND" | "OR") =>
                        {
                            operator_index += 1
                        }
                        _ => break,
                    }
                }
                tokens.push(group_trees[operator_index].to_string());
                tokenize_propositional_logic(
                    group_trees[..operator_index].iter().cloned().collect(),
                    tokens,
                );
                tokenize_propositional_logic(
                    group_trees[operator_index + 1..].iter().cloned().collect(),
                    tokens,
                );
            }
            TokenTree::Ident(ident) => tokens.push(ident.to_string()),
            _ => (),
        }
    }
}
