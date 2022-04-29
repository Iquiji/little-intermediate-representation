use std::{collections::HashMap, rc::Rc};

use little_parser::{Expression, Parser, Programm};

#[derive(Debug, Clone)]
enum LinearInstruction {
    Assign {
        identifier: StaticIdentifier,
        from_reg: Register,
        scope: Scope,
    },
    Call {
        output_reg: Register,
        function_pointer: InitializedFunctionPointer,
        arguments: Vec<DataPointer>,
    },
    Lookup {
        identifier: StaticIdentifier,
        to_reg: Register,
        scope: Scope,
    },
    Cond {
        condition: DataPointer,
        branc_if_true: Branch,
    },
    Return {
        value: Register,
    },
    InitializeFunctionPointer {
        function: LinearBlock,
        from_scope: Scope,
    },
}

#[derive(Debug, Clone)]
struct LinearBlock {
    ident: String,
    program: Vec<LinearInstruction>,
}

#[derive(Debug, Clone)]
struct Register {
    virtual_ident: String,
}

#[derive(Debug, Clone)]
struct InitializedFunctionPointer {
    // Stored in Translator Hashmap for now
    actual_func: String,
}

#[derive(Debug, Clone)]
struct DataPointer {}

#[derive(Debug, Clone)]
enum Scope {
    Global,
    Current,
    Custom(Register),
}

#[derive(Debug, Clone)]
struct StaticIdentifier {
    ident: String,
}

#[derive(Debug, Clone)]
struct Branch {
    program: Vec<LinearInstruction>,
}

pub struct Translator {
    register_counter: usize,
    lambda_map: HashMap<String, LinearBlock>,
    static_data: Vec<StaticData>,
}
impl Translator {
    pub fn default() -> Translator {
        Translator {
            register_counter: 0,
            lambda_map: HashMap::new(),
            static_data: vec![],
        }
    }
    fn ast_to_intermediate_representation(&mut self, ast: Programm) -> LinearBlock {
        unimplemented!()
    }
    fn expr_to_instructions(&mut self, expr: Expression) -> Vec<LinearInstruction> {
        let mut instr_buf = vec![];
        match expr {
            Expression::Quote(quoted) => {
                // Quoted should be static
            }
            Expression::Lambda(formals, body) => {
                // Assign formals to current Scope and then execute body untill last which is returned
            }
            Expression::Cond(cases) => {
                // Conditions and Branches if true
            }
            Expression::Define(global_ident, body) => {
                // Assign to global Scope whater is the body
                let body_instr = &self.expr_to_instructions(
                    std::rc::Rc::<little_parser::Expression>::try_unwrap(body).unwrap(),
                );

                // instr_buf.push(LinearInstruction::Assign {
                //     identifier: (),
                //     from_reg: (),
                //     scope: (),
                // })
            }
            Expression::Let(bindings, body) => {
                // Assign bindings in order and then execute body untill last which is returned in a way
            }
            Expression::LambdaCall(to_call, arguments) => {
                // Call to_call (if ident -> lookup in scope,if lambda -> Direct)
            }
            Expression::Atom(atom) => {
                // Idk is this even possible
                unimplemented!();
            }
            Expression::Identifier(ident) => {
                // Is this possible - prob yes
                unimplemented!();
            }
        }
        instr_buf
    }
}

#[derive(Debug, Clone)]
enum StaticData {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
