use crate::ir::{IR, Type, Program, Function};

pub struct CodeGenerator;

impl CodeGenerator {
    pub fn compile_to_rzb(program: Program) -> String {
        let mut compiled_functions = Vec::new();
        for func in program.functions {
            let mut new_body = Vec::new();
            for ir in func.body {
                match ir {
                    IR::Assign { ref name, .. } => {
                        new_body.push(IR::PushLRS { name: name.clone(), ty: Type::Int32 });
                        new_body.push(ir.clone());
                    }
                    _ => new_body.push(ir),
                }
            }
            compiled_functions.push(Function { body: new_body, ..func });
        }
        serde_json::to_string_pretty(&Program { functions: compiled_functions }).unwrap()
    }
}
