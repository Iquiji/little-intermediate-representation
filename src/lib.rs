use std::collections::HashMap;

use little_parser::{Expression, Parser, Programm};

#[derive(Debug, Clone)]
pub enum LinearInstruction {
    // Need new list building instructions // one for init and one for adding!
    // TODO: rethink register saving?
    // Stack push pop for saving registers!
    // Inneficient but i dont care!
    NewScopeAttachedToAndReplacingCurrent,
    PopScopeAndReplaceWithUpper,
    PushToStack {
        register: Register,
    },
    PopFromStack {
        register: Register,
    },
    LinkedListInit {
        output_reg: Register,
    },
    LinkedListAdd {
        linked_list_reg: Register,
        input_reg: Register,
    },
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
        arguments: Register, // TODO: cant do this! we need some intermediate building to make these! // Maybe pointer to something always list?
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
    // Prob just a series of applying expr_to_instructions
    pub fn ast_to_intermediate_representation(&mut self, ast: Programm) -> LinearBlock {
        unimplemented!()
    }
    /// Design Note!:
    /// Final Data is always pushed onto the stack :)
    pub fn expr_to_instructions(&mut self, expr: Expression) -> Vec<LinearInstruction> {
        let mut instr_buf = vec![];
        match expr {
            Expression::Quote(quoted) => {
                // Quoted should be static
                // Need to convert to StaticRef
            }
            Expression::Lambda(formals, body) => {
                // Assign formals to current Scope and then execute body untill last which is returned
                // This just initializes the LambdaFunction! => Make new Block for Lambda
                // Save formals names for later in StaticRef
                // Body can be same as Let... 
            }
            Expression::Cond(cases) => {
                // Conditions and Branches if true
            }
            Expression::Define(global_ident, body) => {
                // Assign to global Scope whater is the body
                let body_instr = &self.expr_to_instructions(
                    std::rc::Rc::<little_parser::Expression>::try_unwrap(body).unwrap(),
                );
                instr_buf.extend_from_slice(body_instr);

                let static_ref = StaticRef {
                    refname: self.make_static_name(),
                    reftype: StaticData::String(global_ident),
                };
                self.static_data
                    .insert(static_ref.refname.clone(), static_ref.reftype.clone());

                let reg_with_data_assigned = self.make_reg_name();
                instr_buf.push(LinearInstruction::PopFromStack {
                    register: reg_with_data_assigned.clone(),
                });

                instr_buf.push(LinearInstruction::Assign {
                    identifier: static_ref,
                    from_reg: reg_with_data_assigned.clone(),
                    scope: Scope::Current,
                });
                // What do we push onto the stack?! what if we define in a function? we destroy the stack balance??
                // Just return the defines result xD
                instr_buf.push(LinearInstruction::PushToStack {
                    register: reg_with_data_assigned,
                });
            }
            Expression::Let(bindings, body) => {
                // Assign bindings in order and then execute body untill last which is returned in a way

                // Need to build new scope we push into!:
                instr_buf.push(LinearInstruction::NewScopeAttachedToAndReplacingCurrent);

                for binding in bindings {
                    instr_buf.extend_from_slice(&self.expr_to_instructions(binding.1));
                    let data_reg = self.make_reg_name();
                    instr_buf.push(LinearInstruction::PopFromStack{ register: data_reg.clone() });

                    let static_ref = StaticRef {
                        refname: self.make_static_name(),
                        reftype: StaticData::String(binding.0.clone()),
                    };
                    self.static_data
                        .insert(static_ref.refname.clone(), static_ref.reftype.clone());

                    instr_buf.push(LinearInstruction::Assign {
                        identifier: static_ref,
                        from_reg: data_reg,
                        scope: Scope::Current,
                    });
                }
                let body_res_reg = self.make_reg_name();
                for body_expr in body.iter().enumerate(){
                    instr_buf.extend_from_slice(&self.expr_to_instructions(body_expr.1.clone()));
                    instr_buf.push(LinearInstruction::PopFromStack{
                        register: body_res_reg.clone(),
                    });
                }
                instr_buf.push(LinearInstruction::PushToStack{
                    register: body_res_reg,
                });
                // Finally clean new Scope
                instr_buf.push(LinearInstruction::PopScopeAndReplaceWithUpper)
            }
            Expression::LambdaCall(to_call, arguments) => {
                let to_call = std::rc::Rc::try_unwrap(to_call).unwrap();
                // Call to_call (if ident -> lookup in scope,if lambda -> Direct)
                if let Expression::Identifier(ident) = to_call {
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
                    instr_buf.push(LinearInstruction::PushToStack {
                        register: shared_reg,
                    });
                    let args_list = self.make_reg_name();
                    instr_buf.push(LinearInstruction::LinkedListInit {
                        output_reg: args_list.clone(),
                    });
                    instr_buf.push(LinearInstruction::PushToStack {
                        register: args_list,
                    });

                    // For Arguments we can add them to the scope we push into?
                    // How difficult is it to clone?
                    // Do we know the functions aliases?

                    // Converting to a linked list and a function pointer ontop of the stack
                    instr_buf.extend_from_slice(&self.body_to_instruction_list_with_list_to_pop_from_stack_first_in_stack_is_linked_list(arguments));

                    let args_list = self.make_reg_name();
                    instr_buf.push(LinearInstruction::PopFromStack {
                        register: args_list.clone(),
                    });
                    let function_pointer = self.make_reg_name();
                    instr_buf.push(LinearInstruction::LinkedListInit {
                        output_reg: function_pointer.clone(),
                    });
                    let output_reg = self.make_reg_name();
                    // Then call
                    instr_buf.push(LinearInstruction::Call {
                        output_reg,
                        function_pointer,
                        arguments: args_list.clone(),
                    });
                    instr_buf.push(LinearInstruction::PushToStack {
                        register: args_list,
                    });
                } else if let Expression::Lambda(formals, body) = to_call {
                    // Build InitializedPointer First
                    instr_buf.extend_from_slice(
                        &self.expr_to_instructions(Expression::Lambda(formals, body)),
                    );

                    instr_buf.extend_from_slice(&self.body_to_instruction_list_with_list_to_pop_from_stack_first_in_stack_is_linked_list(arguments));

                    // Call func with the arg list and func pointer pushed onto the stack earlier
                    let args_list = self.make_reg_name();
                    instr_buf.push(LinearInstruction::PopFromStack {
                        register: args_list.clone(),
                    });
                    let function_pointer = self.make_reg_name();
                    instr_buf.push(LinearInstruction::LinkedListInit {
                        output_reg: function_pointer.clone(),
                    });
                    let output_reg = self.make_reg_name();
                    // Then call
                    instr_buf.push(LinearInstruction::Call {
                        output_reg,
                        function_pointer,
                        arguments: args_list.clone(),
                    });
                    instr_buf.push(LinearInstruction::PushToStack {
                        register: args_list,
                    });
                } else if let Expression::LambdaCall(formals, body) = to_call {
                    instr_buf.extend_from_slice(
                        &self.expr_to_instructions(Expression::LambdaCall(formals, body)),
                    );

                    instr_buf.extend_from_slice(&self.body_to_instruction_list_with_list_to_pop_from_stack_first_in_stack_is_linked_list(arguments));

                    // Call func with the arg list and func pointer pushed onto the stack earlier
                    let args_list = self.make_reg_name();
                    instr_buf.push(LinearInstruction::PopFromStack {
                        register: args_list.clone(),
                    });
                    let function_pointer = self.make_reg_name();
                    instr_buf.push(LinearInstruction::LinkedListInit {
                        output_reg: function_pointer.clone(),
                    });
                    let output_reg = self.make_reg_name();
                    // Then call
                    instr_buf.push(LinearInstruction::Call {
                        output_reg,
                        function_pointer,
                        arguments: args_list.clone(),
                    });
                    instr_buf.push(LinearInstruction::PushToStack {
                        register: args_list,
                    });
                } else {
                    // Should be correct?!
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
    fn body_to_instruction_list_with_list_to_pop_from_stack_first_in_stack_is_linked_list(
        &mut self,
        body: Vec<Expression>,
    ) -> Vec<LinearInstruction> {
        let mut instr_buf = vec![];
        for arg in body {
            instr_buf.extend_from_slice(&self.expr_to_instructions(arg));

            let to_add_reg = self.make_reg_name();
            instr_buf.push(LinearInstruction::PopFromStack {
                register: to_add_reg.clone(),
            });
            let args_list = self.make_reg_name();
            instr_buf.push(LinearInstruction::LinkedListInit {
                output_reg: args_list.clone(),
            });
            instr_buf.push(LinearInstruction::LinkedListAdd {
                linked_list_reg: args_list.clone(),
                input_reg: to_add_reg.clone(),
            });
            instr_buf.push(LinearInstruction::PushToStack {
                register: args_list.clone(),
            });
        }
        instr_buf
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
