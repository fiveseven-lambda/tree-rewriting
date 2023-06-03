use nom::{
    branch::alt,
    character::complete::{alpha1, char, u32},
    combinator::{map, opt},
    multi::separated_list0,
    sequence::{delimited, pair},
};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    io,
};

fn main() {
    let stdin = io::stdin();
    let mut rules = Vec::new();
    let read_line = || {
        let mut s = String::new();
        stdin.read_line(&mut s).unwrap();
        s
    };
    let mut tree = loop {
        match read_line().trim() {
            "rule" => {
                let s = read_line();
                let (_, from) = parse_pattern(s.trim()).unwrap();
                let s = read_line();
                let (_, to) = parse_pattern(s.trim()).unwrap();
                rules.push((from, to));
            }
            _ => {
                let s = read_line();
                let (_, tree) = parse_tree(s.trim()).unwrap();
                break tree;
            }
        }
    };
    for (i, (from, to)) in rules.iter().enumerate() {
        println!("rule {i}: {from} -> {to}");
    }
    println!("{tree}");
    loop {
        let s = read_line();
        let s = s.trim();
        if s.is_empty() {
            break;
        }
        let i: usize = s.parse().unwrap();
        let (from, to) = &rules[i];
        let mut vars = HashMap::new();
        assert!(identify(tree, &from, &mut vars));
        tree = subst(to, &vars);
        println!("{tree}");
    }
}

#[derive(Debug, Clone)]
struct Tree {
    value: u32,
    children: Vec<Tree>,
}

impl Display for Tree {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)?;
        if !self.children.is_empty() {
            write!(
                f,
                "[{}]",
                self.children
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            )?;
        }
        Ok(())
    }
}
fn parse_tree(s: &str) -> nom::IResult<&str, Tree> {
    map(
        pair(
            u32,
            map(
                opt(delimited(
                    char('['),
                    separated_list0(char(','), parse_tree),
                    char(']'),
                )),
                |list| list.unwrap_or_else(Vec::new),
            ),
        ),
        |(value, children)| Tree { value, children },
    )(s)
}

#[derive(Debug)]
enum Pattern {
    Var(String),
    Node { value: u32, children: Vec<Pattern> },
}
fn parse_pattern(s: &str) -> nom::IResult<&str, Pattern> {
    let var = map(alpha1, |s: &str| Pattern::Var(s.to_string()));
    let node = map(
        pair(
            u32,
            map(
                opt(delimited(
                    char('['),
                    separated_list0(char(','), parse_pattern),
                    char(']'),
                )),
                |list| list.unwrap_or_else(Vec::new),
            ),
        ),
        |(value, children)| Pattern::Node { value, children },
    );
    alt((var, node))(s)
}
fn subst(pat: &Pattern, vars: &HashMap<String, Tree>) -> Tree {
    match pat {
        Pattern::Var(name) => vars[name].clone(),
        &Pattern::Node {
            value,
            ref children,
        } => {
            let children = children.iter().map(|pat| subst(pat, vars)).collect();
            Tree { value, children }
        }
    }
}
fn identify(tree: Tree, pat: &Pattern, vars: &mut HashMap<String, Tree>) -> bool {
    match pat {
        Pattern::Var(name) => {
            vars.insert(name.clone(), tree);
            true
        }
        &Pattern::Node {
            value,
            ref children,
        } => {
            if tree.value == value {
                tree.children.len() == children.len()
                    && tree
                        .children
                        .into_iter()
                        .zip(children)
                        .all(|(tree, pat)| identify(tree, pat, vars))
            } else {
                false
            }
        }
    }
}
impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Var(name) => write!(f, "{name}"),
            Pattern::Node { value, children } => {
                write!(f, "{}", value)?;
                if !children.is_empty() {
                    write!(
                        f,
                        "[{}]",
                        children
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(",")
                    )?;
                }
                Ok(())
            }
        }
    }
}
