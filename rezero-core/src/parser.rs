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

stmt = { let_stmt | assign_stmt | if_stmt | return_stmt }
let_stmt = { "let" ~ ident ~ ":" ~ ty ~ "=" ~ expr ~ ";" }
assign_stmt = { ident ~ "=" ~ expr ~ ";" }
if_stmt = { "if" ~ "(" ~ expr ~ ")" ~ "{" ~ stmt* ~ "}" ~ ("else" ~ "{" ~ stmt* ~ "}")? }
return_stmt = { "return" ~ expr ~ ";" }

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

pub fn parse(input: &str) -> anyhow::Result<Program> {
    let pairs = ReZeroParser::parse(Rule::program, input).map_err(|e| anyhow::anyhow!(e))?;
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
    let mut ret_ty = Type::Int;
    let mut body = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::annotation => {
                let mut it = inner.into_inner();
                let ann = it.next().unwrap().as_str();
                let expr = parse_expr(it.next().unwrap());
                if ann == "Requires" { body.push(IR::AssertPre(expr)); }
                else if ann == "Ensures" { body.push(IR::AssertPost(expr)); }
            }
            Rule::ident => name = inner.as_str().to_string(),
            Rule::params => {
                for p in inner.into_inner() {
                    let mut pit = p.into_inner();
                    params.push((pit.next().unwrap().as_str().to_string(), parse_ty(pit.next().unwrap())));
                }
            }
            Rule::ty => ret_ty = parse_ty(inner),
            Rule::stmt => body.push(parse_stmt(inner)),
            _ => {}
        }
    }
    Function { name, params, ret_ty, body }
}

fn parse_ty(pair: pest::iterators::Pair<Rule>) -> Type {
    match pair.as_str() { "int" => Type::Int, "bool" => Type::Bool, _ => Type::Int }
}

fn parse_stmt(pair: pest::iterators::Pair<Rule>) -> IR {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::let_stmt => {
            let mut it = inner.into_inner();
            let name = it.next().unwrap().as_str().to_string();
            let ty = parse_ty(it.next().unwrap());
            let init = parse_expr(it.next().unwrap());
            IR::Declare { name, ty, init: Some(init) }
        }
        Rule::assign_stmt => {
            let mut it = inner.into_inner();
            IR::Assign { name: it.next().unwrap().as_str().to_string(), val: parse_expr(it.next().unwrap()) }
        }
        Rule::if_stmt => {
            let mut it = inner.into_inner();
            let cond = parse_expr(it.next().unwrap());
            let mut then_b = Vec::new();
            for s in it.next().unwrap().into_inner() { then_b.push(parse_stmt(s)); }
            let mut else_b = Vec::new();
            if let Some(eb) = it.next() {
                for s in eb.into_inner() { else_b.push(parse_stmt(s)); }
            }
            IR::If { cond, then_b, else_b }
        }
        Rule::return_stmt => IR::Return(parse_expr(inner.into_inner().next().unwrap())),
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
                ">" => CompOp::Gt, ">=" => CompOp::Ge, "<" => CompOp::Lt, "<=" => CompOp::Le, "==" => CompOp::Eq, "!=" => CompOp::Ne, _ => unreachable!(),
            };
            Expr::Compare { op, left: Box::new(left), right: Box::new(parse_expr(it.next().unwrap())) }
        }
        Rule::term => {
            let mut it = pair.into_inner();
            let mut res = parse_expr(it.next().unwrap());
            while let Some(op_pair) = it.next() {
                let op = match op_pair.as_str() { "+" => Op::Add, "-" => Op::Sub, "*" => Op::Mul, "/" => Op::Div, _ => unreachable!(), };
                res = Expr::Binary { op, left: Box::new(res), right: Box::new(parse_expr(it.next().unwrap())) };
            }
            res
        }
        Rule::factor => {
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::ident => Expr::Variable(inner.as_str().to_string()),
                Rule::literal => Expr::Literal(inner.as_str().parse().unwrap()),
                Rule::expr => parse_expr(inner),
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}
