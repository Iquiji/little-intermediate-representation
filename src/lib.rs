use std::{collections::HashMap, rc::Rc};

use little_parser::{Expression, Parser, Programm};

#[derive(Debug, Clone)]
pub enum LinearInstruction {
    // Need new list building instructions // one for init and one for adding!
    // TODO: rethink register saving?
    Assign {
        identifier: StaticRef,
        from_reg: Register,
        scope: Scope,
    },
    // We need to deepclone the scope when we call tho!
    Call {
        output_reg: Register,
        /// Originaly InitializedFunctionPointer but that only adds unneded complexity without gain
        function_pointer: Register,
        arguments: Vec<Pointer>, // TODO: cant do this! we need some intermediate building to make these! // Maybe pointer to something always list?
    },
    Lookup {
        identifier: StaticRef,
        to_reg: Register,
        scope: Scope,
    },
    Cond {
        /// Pointer
        condition: Register,
        branc_if_true: Branch,
    },
    Return {
        value: Register,
    },
    InitializeFunctionPointer {
        function: LinearBlock,
        from_scope: Scope,
        outpu_reg: Register,
    },
}

#[derive(Debug, Clone)]
pub struct LinearBlock {
    ident: String,
    program: Vec<LinearInstruction>,
}

#[derive(Debug, Clone)]
pub struct Register {
    virtual_ident: String,
}

#[derive(Debug, Clone)]
pub struct InitializedFunctionPointer {
    // Stored in Translator Hashmap for now
    actual_func: String,
}

#[derive(Debug, Clone)]
pub enum Pointer {
    StaticData(StaticRef),
    DynamicPointer,
}

#[derive(Debug, Clone)]
pub struct StaticRef {
    refname: String,
    reftype: StaticData,
}

#[derive(Debug, Clone)]
pub enum Scope {
    Global,
    Current,
    Custom(Register),
}

#[derive(Debug, Clone)]
pub struct Branch {
    program: Vec<LinearInstruction>,
}

pub struct Translator {
    register_counter: usize,
    static_data_counter: usize,
    static_data: HashMap<String, StaticData>,
    lambda_map: HashMap<String, LinearBlock>,
}
impl Translator {
    pub fn default() -> Translator {
        Translator {
            register_counter: 0,
            static_data_counter: 0,
            lambda_map: HashMap::new(),
            static_data: HashMap::new(),
        }
    }
    pub fn ast_to_intermediate_representation(&mut self, ast: Programm) -> LinearBlock {
        unimplemented!()
    }
    pub fn expr_to_instructions(&mut self, expr: Expression) -> Vec<LinearInstruction> {
        let mut instr_buf = vec![];
        match expr {
            Expression::Quote(quoted) => {
                // Quoted should be static
            }
            Expression::Lambda(formals, body) => {
                // Assign formals to current Scope and then execute body untill last which is returned
                // This just initializes the LambdaFunction! => Make new Block for Lambda
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
                if let Expression::Identifier(ident) =
                    std::rc::Rc::try_unwrap(to_call.clone()).unwrap()
                {
                    let shared_reg = self.make_reg_name();
                    let static_name = self.make_static_name();
                    self.static_data
                        .insert(static_name.clone(), StaticData::Identifier(ident.clone()));
                    // Lookup
                    instr_buf.push(LinearInstruction::Lookup {
                        identifier: StaticRef {
                            refname: static_name,
                            reftype: StaticData::Identifier(ident),
                        },
                        to_reg: shared_reg.clone(),
                        scope: Scope::Current,
                    });
                    // For Arguments we can add them to the scope we push into?
                    // How difficult is it to clone?
                    // Do we know the functions aliases?

                    // Then call
                    instr_buf.push(LinearInstruction::Call {
                        output_reg: (),
                        function_pointer: shared_reg.clone(),
                        arguments: (),
                    })
                } else if let Expression::Lambda(formals, body) =
                    std::rc::Rc::try_unwrap(to_call).unwrap()
                {
                    // Build InitializedPointer First
                } else {
                    // Untrue! Lambda call can return func as well
                    unreachable!();
                }
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
    fn make_reg_name(&mut self) -> Register {
        let temp = Register {
            virtual_ident: "r".to_owned() + &self.register_counter.to_string(),
        };
        self.register_counter += 1;
        temp
    }
    fn make_static_name(&mut self) -> String {
        let temp = "static".to_owned() + &self.static_data_counter.to_string();
        self.static_data_counter += 1;
        temp
    }
}

#[derive(Debug, Clone)]
enum StaticData {
    Bool(bool),
    Integer(i32),
    String(String),
    Identifier(String),
    List(Vec<StaticData>),
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
