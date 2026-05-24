use crate::ir::{IR, Expr, Op, CompOp, Function};
use z3::{Config, Context, Solver, Params, ast::{Int, Bool, Ast}, SatResult};
use std::collections::HashMap;

pub struct Verifier<'ctx> {
    ctx: &'ctx Context,
    solver: Solver<'ctx>,
}

impl<'ctx> Verifier<'ctx> {
    pub fn new(ctx: &'ctx Context) -> Self {
        let solver = Solver::new(ctx);
        let mut params = Params::new(ctx);
        params.set_u32("timeout", 5000);
        solver.set_params(&params);
        Self { ctx, solver }
    }

    fn translate_expr(&self, expr: &Expr, vars: &HashMap<String, Int<'ctx>>) -> Result<Int<'ctx>, String> {
        match expr {
            Expr::Literal(val) => Ok(Int::from_i64(self.ctx, *val as i64)),
            Expr::Variable(name) => vars.get(name).cloned().ok_or_else(|| format!("Undefined variable: {}", name)),
            Expr::Binary { op, left, right } => {
                let l = self.translate_expr(left, vars)?;
                let r = self.translate_expr(right, vars)?;
                match op {
                    Op::Add => Ok(l + r),
                    Op::Sub => Ok(l - r),
                    Op::Mul => Ok(l * r),
                    Op::Div => {
                        let is_zero = r._eq(&Int::from_i64(self.ctx, 0));
                        self.solver.assert(&is_zero.not());
                        Ok(l / r)
                    }
                }
            }
            _ => Ok(Int::from_i64(self.ctx, 0)),
        }
    }

    fn translate_compare(&self, expr: &Expr, vars: &HashMap<String, Int<'ctx>>) -> Result<Bool<'ctx>, String> {
        match expr {
            Expr::Compare { op, left, right } => {
                let l = self.translate_expr(left, vars)?;
                let r = self.translate_expr(right, vars)?;
                match op {
                    CompOp::Gt => Ok(l.gt(&r)), CompOp::Ge => Ok(l.ge(&r)),
                    CompOp::Lt => Ok(l.lt(&r)), CompOp::Le => Ok(l.le(&r)),
                    CompOp::Eq => Ok(l._eq(&r)), CompOp::Ne => Ok(l._eq(&r).not()),
                }
            }
            Expr::BoolLiteral(b) => Ok(Bool::from_bool(self.ctx, *b)),
            _ => Err("Invalid comparison".to_string()),
        }
    }

    pub fn verify_function(&mut self, func: &Function) -> Result<(), String> {
        self.solver.push();
        let mut vars = HashMap::new();
        let mut version = HashMap::new();

        for (name, _) in &func.params {
            let vname = format!("{}_0", name);
            vars.insert(name.clone(), Int::new_const(self.ctx, vname.as_str()));
            version.insert(name.clone(), 0);
        }

        for req in &func.requires {
            let cond = self.translate_compare(req, &vars)?;
            self.solver.assert(&cond);
        }

        self.verify_body(&func.body, &mut vars, &mut version)?;

        for ens in &func.ensures {
            let cond = self.translate_compare(ens, &vars)?;
            self.solver.push();
            self.solver.assert(&cond.not());
            if self.solver.check() == SatResult::Sat {
                let model = self.solver.get_model().unwrap();
                return Err(format!("Post-condition violated. Counter-example: {:?}", model));
            }
            self.solver.pop(1);
        }

        self.solver.pop(1);
        Ok(())
    }

    fn verify_body(&mut self, body: &[IR], vars: &mut HashMap<String, Int<'ctx>>, version: &mut HashMap<String, u32>) -> Result<(), String> {
        for ir in body {
            match ir {
                IR::Declare { name, init_val, .. } => {
                    let v = version.entry(name.clone()).or_insert(0);
                    let vname = format!("{}_{}", name, v);
                    let z3_var = Int::new_const(self.ctx, vname.as_str());
                    if let Some(val_expr) = init_val {
                        let val = self.translate_expr(val_expr, vars)?;
                        self.solver.assert(&z3_var._eq(&val));
                    }
                    vars.insert(name.clone(), z3_var);
                }
                IR::Assign { name, val } => {
                    let v = version.entry(name.clone()).or_insert(0);
                    *v += 1;
                    let vname = format!("{}_{}", name, v);
                    let z3_var = Int::new_const(self.ctx, vname.as_str());
                    let z3_val = self.translate_expr(val, vars)?;
                    self.solver.assert(&z3_var._eq(&z3_val));
                    vars.insert(name.clone(), z3_var);
                }
                IR::If { cond, then_branch, else_branch } => {
                    // Simplified if verification
                    let c = self.translate_compare(cond, vars)?;
                    self.solver.push();
                    self.solver.assert(&c);
                    self.verify_body(then_branch, &mut vars.clone(), &mut version.clone())?;
                    self.solver.pop(1);
                    self.solver.push();
                    self.solver.assert(&c.not());
                    self.verify_body(else_branch, &mut vars.clone(), &mut version.clone())?;
                    self.solver.pop(1);
                }
                IR::Assert { cond, message } => {
                    let c = self.translate_compare(cond, vars)?;
                    self.solver.push();
                    self.solver.assert(&c.not());
                    if self.solver.check() == SatResult::Sat {
                        return Err(format!("Assertion failed: {}", message));
                    }
                    self.solver.pop(1);
                }
                _ => {}
            }
        }
        Ok(())
    }
}
