use pest::Parser;
use pest_derive::Parser;
use crate::ir::*;

#[derive(Parser)]
#[grammar_inline = r#"
WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
COMMENT = _{ "//" ~ (!"\n" ~ ANY)* }

program = { SOI ~ function* ~ EOI }
function = { annotation* ~ "fn" ~ ident ~ "(" ~ params? ~ ")" ~ "->" ~ ty ~ "{" ~ stmt* ~ "}" }
annotation = { "@" ~ ident ~ "(" ~ expr ~ ")" }
params = { param ~ ("," ~ param)* }
param = { ident ~ ":" ~ ty }
ty = { "int" | "bool" }

stmt = { let_stmt | assign_stmt | return_stmt | if_stmt }
let_stmt = { "let" ~ ident ~ ":" ~ ty ~ "=" ~ expr ~ ";" }
assign_stmt = { ident ~ "=" ~ expr ~ ";" }
return_stmt = { "return" ~ expr ~ ";" }
if_stmt = { "if" ~ "(" ~ expr ~ ")" ~ "{" ~ stmt* ~ "}" ~ ("else" ~ "{" ~ stmt* ~ "}")? }

expr = { comparison | term }
comparison = { term ~ comp_op ~ term }
comp_op = { ">=" | ">" | "<=" | "<" | "==" | "!=" }
term = { factor ~ (bin_op ~ factor)* }
bin_op = { "+" | "-" | "*" | "/" }
factor = { literal | ident | "(" ~ expr ~ ")" }

ident = @{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* }
literal = @{ "-"? ~ ASCII_DIGIT+ }
"#]
pub struct ReZeroParser;

pub fn parse_rz(input: &str) -> Result<Program, String> {
    let pairs = ReZeroParser::parse(Rule::program, input).map_err(|e| e.to_string())?;
    let mut functions = Vec::new();

    for pair in pairs {
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::function {
                functions.push(parse_function(inner));
            }
        }
    }

    Ok(Program { functions })
}

fn parse_function(pair: pest::iterators::Pair<Rule>) -> Function {
    let mut name = String::new();
    let mut params = Vec::new();
    let mut return_type = Type::Int32;
    let mut requires = Vec::new();
    let mut ensures = Vec::new();
    let mut body = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::annotation => {
                let mut it = inner.into_inner();
                let ann_name = it.next().unwrap().as_str();
                let expr = parse_expr(it.next().unwrap());
                match ann_name {
                    "Requires" => requires.push(expr),
                    "Ensures" => ensures.push(expr),
                    _ => {}
                }
            }
            Rule::ident => name = inner.as_str().to_string(),
            Rule::params => {
                for p in inner.into_inner() {
                    let mut pit = p.into_inner();
                    let pname = pit.next().unwrap().as_str().to_string();
                    let pty = parse_ty(pit.next().unwrap());
                    params.push((pname, pty));
                }
            }
            Rule::ty => return_type = parse_ty(inner),
            Rule::stmt => body.push(parse_stmt(inner)),
            _ => {}
        }
    }

    Function { name, params, return_type, requires, ensures, body }
}

fn parse_ty(pair: pest::iterators::Pair<Rule>) -> Type {
    match pair.as_str() {
        "int" => Type::Int32,
        "bool" => Type::Bool,
        _ => Type::Int32,
    }
}

fn parse_stmt(pair: pest::iterators::Pair<Rule>) -> IR {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::let_stmt => {
            let mut it = inner.into_inner();
            let name = it.next().unwrap().as_str().to_string();
            let ty = parse_ty(it.next().unwrap());
            let val = parse_expr(it.next().unwrap());
            IR::Declare { name, ty, init_val: Some(val) }
        }
        Rule::assign_stmt => {
            let mut it = inner.into_inner();
            let name = it.next().unwrap().as_str().to_string();
            let val = parse_expr(it.next().unwrap());
            IR::Assign { name, val }
        }
        Rule::return_stmt => {
            let val = parse_expr(inner.into_inner().next().unwrap());
            IR::Return(val)
        }
        Rule::if_stmt => {
            let mut it = inner.into_inner();
            let cond = parse_expr(it.next().unwrap());
            let mut then_branch = Vec::new();
            let mut else_branch = Vec::new();
            let then_block = it.next().unwrap();
            for s in then_block.into_inner() {
                then_branch.push(parse_stmt(s));
            }
            if let Some(else_block) = it.next() {
                for s in else_block.into_inner() {
                    else_branch.push(parse_stmt(s));
                }
            }
            IR::If { cond, then_branch, else_branch }
        }
        _ => unreachable!(),
    }
}

fn parse_expr(pair: pest::iterators::Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::expr => parse_expr(pair.into_inner().next().unwrap()),
        Rule::comparison => {
            let mut it = pair.into_inner();
            let left = parse_expr(it.next().unwrap());
            let op = match it.next().unwrap().as_str() {
                ">" => CompOp::Gt, ">=" => CompOp::Ge,
                "<" => CompOp::Lt, "<=" => CompOp::Le,
                "==" => CompOp::Eq, "!=" => CompOp::Ne,
                _ => unreachable!(),
            };
            let right = parse_expr(it.next().unwrap());
            Expr::Compare { op, left: Box::new(left), right: Box::new(right) }
        }
        Rule::term => {
            let mut it = pair.into_inner();
            let mut res = parse_expr(it.next().unwrap());
            while let Some(op_pair) = it.next() {
                let op = match op_pair.as_str() {
                    "+" => Op::Add, "-" => Op::Sub, "*" => Op::Mul, "/" => Op::Div,
                    _ => unreachable!(),
                };
                let right = parse_expr(it.next().unwrap());
                res = Expr::Binary { op, left: Box::new(res), right: Box::new(right) };
            }
            res
        }
        Rule::factor => parse_expr(pair.into_inner().next().unwrap()),
        Rule::ident => Expr::Variable(pair.as_str().to_string()),
        Rule::literal => Expr::Literal(pair.as_str().parse().unwrap()),
        _ => unreachable!(),
    }
}
