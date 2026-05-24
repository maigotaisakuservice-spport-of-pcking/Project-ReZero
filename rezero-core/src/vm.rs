use crate::ir::{Program, IR, Expr, Op};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VMState {
    pub pc: usize,
    pub variables: HashMap<String, i32>,
    pub lrs_stack: Vec<(String, i32)>,
}

pub struct ReversibleVM {
    pub program: Program,
    pub state: VMState,
    pub history: Vec<VMState>,
}

impl ReversibleVM {
    pub fn new(program: Program) -> Self {
        Self {
            program,
            state: VMState { pc: 0, variables: HashMap::new(), lrs_stack: Vec::new() },
            history: Vec::new(),
        }
    }

    fn eval_expr(&self, expr: &Expr) -> i32 {
        match expr {
            Expr::Literal(v) => *v,
            Expr::Variable(name) => *self.state.variables.get(name).unwrap_or(&0),
            Expr::Binary { op, left, right } => {
                let l = self.eval_expr(left);
                let r = self.eval_expr(right);
                match op {
                    Op::Add => l + r, Op::Sub => l - r,
                    Op::Mul => l * r, Op::Div => if r != 0 { l / r } else { 0 },
                }
            }
            _ => 0,
        }
    }

    pub fn step_forward(&mut self) -> bool {
        if self.program.functions.is_empty() { return false; }
        let func = &self.program.functions[0];
        if self.state.pc >= func.body.len() { return false; }

        self.history.push(self.state.clone());
        let ir = &func.body[self.state.pc];
        match ir {
            IR::Declare { name, init_val, .. } => {
                let val = init_val.as_ref().map(|e| self.eval_expr(e)).unwrap_or(0);
                self.state.variables.insert(name.clone(), val);
            }
            IR::Assign { name, val } => {
                let v = self.eval_expr(val);
                self.state.variables.insert(name.clone(), v);
            }
            IR::PushLRS { name, .. } => {
                let val = *self.state.variables.get(name).unwrap_or(&0);
                self.state.lrs_stack.push((name.clone(), val));
            }
            _ => {}
        }
        self.state.pc += 1;
        true
    }

    pub fn step_backward(&mut self) -> bool {
        if let Some(prev_state) = self.history.pop() {
            self.state = prev_state;
            true
        } else {
            false
        }
    }
}
