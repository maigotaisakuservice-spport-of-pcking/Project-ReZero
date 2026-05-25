use crate::ir::*;
use z3::{Config, Context, Solver, Params, ast::{Int, Bool, Ast}, SatResult};
use std::collections::HashMap;

pub struct Verifier<'ctx> { ctx: &'ctx Context, solver: Solver<'ctx> }

impl<'ctx> Verifier<'ctx> {
    pub fn new(ctx: &'ctx Context) -> Self {
        let solver = Solver::new(ctx);
        let mut params = Params::new(ctx);
        params.set_u32("timeout", 5000);
        solver.set_params(&params);
        Self { ctx, solver }
    }

    fn translate_expr(&self, expr: &Expr, vars: &HashMap<String, Int<'ctx>>) -> anyhow::Result<Int<'ctx>> {
        match expr {
            Expr::Literal(v) => Ok(Int::from_i64(self.ctx, *v as i64)),
            Expr::Variable(n) => vars.get(n).cloned().ok_or(anyhow::anyhow!("Undefined: {}", n)),
            Expr::Binary { op, left, right } => {
                let l = self.translate_expr(left, vars)?;
                let r = self.translate_expr(right, vars)?;
                match op {
                    Op::Add => Ok(l + r), Op::Sub => Ok(l - r), Op::Mul => Ok(l * r),
                    Op::Div => {
                        self.solver.assert(&r._eq(&Int::from_i64(self.ctx, 0)).not());
                        Ok(l / r)
                    }
                }
            }
            Expr::ArrayAccess { name, index } => {
                let idx = self.translate_expr(index, vars)?;
                // Bounds check: 0 <= idx < 100 (assuming size 100 for simplicity)
                self.solver.assert(&idx.ge(&Int::from_i64(self.ctx, 0)));
                self.solver.assert(&idx.lt(&Int::from_i64(self.ctx, 100)));
                Ok(Int::new_const(self.ctx, format!("{}_at_idx", name).as_str()))
            }
            Expr::PointerDereference(name) => {
                let p = vars.get(name).ok_or(anyhow::anyhow!("Undefined pointer: {}", name))?;
                self.solver.assert(&p._eq(&Int::from_i64(self.ctx, 0)).not());
                Ok(Int::new_const(self.ctx, format!("val_at_{}", name).as_str()))
            }
            _ => Err(anyhow::anyhow!("Invalid expression")),
        }
    }

    fn translate_bool(&self, expr: &Expr, vars: &HashMap<String, Int<'ctx>>) -> anyhow::Result<Bool<'ctx>> {
        match expr {
            Expr::Compare { op, left, right } => {
                let l = self.translate_expr(left, vars)?;
                let r = self.translate_expr(right, vars)?;
                Ok(match op {
                    CompOp::Gt => l.gt(&r), CompOp::Ge => l.ge(&r), CompOp::Lt => l.lt(&r),
                    CompOp::Le => l.le(&r), CompOp::Eq => l._eq(&r), CompOp::Ne => l._eq(&r).not(),
                })
            }
            _ => Err(anyhow::anyhow!("Expected comparison")),
        }
    }

    pub fn verify_function(&mut self, func: &Function) -> anyhow::Result<()> {
        self.solver.push();
        let mut vars = HashMap::new();
        let mut version = HashMap::new();
        for (name, _) in &func.params {
            vars.insert(name.clone(), Int::new_const(self.ctx, format!("{}_0", name).as_str()));
            version.insert(name.clone(), 0);
        }
        self.verify_stmts(&func.body, &mut vars, &mut version)?;
        self.solver.pop(1);
        Ok(())
    }

    fn verify_stmts(&mut self, stmts: &[IR], vars: &mut HashMap<String, Int<'ctx>>, version: &mut HashMap<String, u32>) -> anyhow::Result<()> {
        for ir in stmts {
            match ir {
                IR::AssertPre(e) => self.solver.assert(&self.translate_bool(e, vars)?),
                IR::AssertPost(e) => {
                    let cond = self.translate_bool(e, vars)?;
                    self.solver.push();
                    self.solver.assert(&cond.not());
                    if self.solver.check() == SatResult::Sat {
                        return Err(anyhow::anyhow!("Post-condition violated. Model: {:?}", self.solver.get_model()));
                    }
                    self.solver.pop(1);
                }
                IR::Declare { name, init, .. } => {
                    let v = version.entry(name.clone()).or_insert(0);
                    let z3_var = Int::new_const(self.ctx, format!("{}_{}", name, v).as_str());
                    if let Some(i) = init {
                        let val = self.translate_expr(i, vars)?;
                        self.solver.assert(&z3_var._eq(&val));
                    }
                    vars.insert(name.clone(), z3_var);
                }
                IR::Assign { name, val } => {
                    let v = version.entry(name.clone()).or_insert(0);
                    *v += 1;
                    let z3_var = Int::new_const(self.ctx, format!("{}_{}", name, v).as_str());
                    let z3_val = self.translate_expr(val, vars)?;
                    self.solver.assert(&z3_var._eq(&z3_val));
                    vars.insert(name.clone(), z3_var);
                }
                IR::If { cond, then_b, else_b } => {
                    let c = self.translate_bool(cond, vars)?;
                    self.solver.push();
                    self.solver.assert(&c);
                    self.verify_stmts(then_b, &mut vars.clone(), &mut version.clone())?;
                    self.solver.pop(1);
                    self.solver.push();
                    self.solver.assert(&c.not());
                    self.verify_stmts(else_b, &mut vars.clone(), &mut version.clone())?;
                    self.solver.pop(1);
                }
                _ => {}
            }
        }
        Ok(())
    }
}
