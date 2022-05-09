use std::{collections::HashMap, vec};

use little_parser::{Expression, Programm};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinearInstruction {
    // Need new list building instructions // one for init and one for adding!
    // TODO: rethink register saving?
    // Stack push pop for saving registers!
    // Inneficient but i dont care!
    // We need a Instruction for accepting formals!
    AcceptToFormals {
        static_formals_list: StaticRef,
    },
    NewScopeAttachedToAndReplacingCurrent,
    PopScopeAndReplaceWithUpper,
    StaticRefToRegister {
        static_ref: StaticRef,
        to_reg: Register,
    },
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
        function: FunctionPointer,
        from_scope: Scope,
        outpu_reg: Register,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinearBlock {
    ident: String,
    program: Vec<LinearInstruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Register {
    virtual_ident: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionPointer {
    // Stored in Translator Hashmap for now
    actual_func: String,
    formals_list: StaticRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pointer {
    StaticData(StaticRef),
    DynamicPointer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticRef {
    refname: String,
    reftype: StaticData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    Global,
    Current,
    Custom(Register),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    program: Vec<LinearInstruction>,
}

#[derive(Debug)]
pub struct Translator {
    register_counter: usize,
    anon_lambda_counter: usize,
    static_data_counter: usize,
    static_data: HashMap<String, StaticData>,
    lambda_map: HashMap<String, LinearBlock>,
}
impl Translator {
    pub fn default() -> Translator {
        Translator {
            register_counter: 0,
            static_data_counter: 0,
            anon_lambda_counter: 0,
            lambda_map: HashMap::new(),
            static_data: HashMap::new(),
        }
    }
    // Prob just a series of applying expr_to_instructions
    pub fn ast_to_intermediate_representation(&mut self, ast: Programm) -> LinearBlock {
        let mut main = LinearBlock {
            ident: "main".into(),
            program: vec![],
        };
        match ast {
            Programm::Expression(inner) => {
                for expr in inner {
                    main.program
                        .extend_from_slice(&self.expr_to_instructions(expr));
                }
            }
        }
        self.lambda_map.insert("main".into(), main.clone());

        main
    }
    /// Design Note!:
    /// Final Data is always pushed onto the stack :)
    pub fn expr_to_instructions(&mut self, expr: Expression) -> Vec<LinearInstruction> {
        let mut instr_buf = vec![];
        match expr {
            Expression::Quote(quoted) => {
                // Quoted should be static
                // Need to convert to StaticRef
                fn atomtype_to_static_data(atom: little_parser::AtomTypes) -> StaticData {
                    match atom {
                        little_parser::AtomTypes::Integer(int) => StaticData::Integer(int),
                        little_parser::AtomTypes::Symbol(symbol) => StaticData::Identifier(symbol),
                        little_parser::AtomTypes::String(string) => StaticData::String(string),
                        little_parser::AtomTypes::Boolean(boolean) => StaticData::Bool(boolean),
                        little_parser::AtomTypes::List(list) => StaticData::List(
                            list.iter()
                                .map(|x| atomtype_to_static_data(x.clone()))
                                .collect(),
                        ),
                    }
                }

                let static_ref_name = self.make_static_name();
                self.static_data.insert(
                    static_ref_name.clone(),
                    atomtype_to_static_data(quoted.clone()),
                );

                let quoted_ref = StaticRef {
                    refname: static_ref_name,
                    reftype: atomtype_to_static_data(quoted),
                };

                // Somehow take the quoted data to a register? => new Command
                let reg = self.make_reg_name();
                instr_buf.push(LinearInstruction::StaticRefToRegister {
                    static_ref: quoted_ref,
                    to_reg: reg.clone(),
                });
                instr_buf.push(LinearInstruction::PushToStack { register: reg });
            }
            Expression::Lambda(formals, body) => {
                // Assign formals to current Scope and then execute body untill last which is returned
                // This just initializes the LambdaFunction! => Make new Block for Lambda
                // Save formals names for later in StaticRef
                // Body can be same as Let...
                // how do we accept the args into the formals?
                // Make accept formals Instruction taking StaticRef and then a reg?
                // We accept formals and push them to scope internally
                let formals_vec = StaticData::List(
                    formals
                        .iter()
                        .map(|formal| StaticData::Identifier(formal.to_string()))
                        .collect(),
                );

                let anon_lambda_name = self.make_anon_lambda_name();
                self.static_data
                    .insert(anon_lambda_name.clone(), formals_vec.clone());

                let formals_vec_ref = StaticRef {
                    refname: anon_lambda_name.clone(),
                    reftype: formals_vec,
                };

                // Init the lambda block with the coresponding name and add content later
                let mut lambda_block = LinearBlock {
                    ident: anon_lambda_name.clone(),
                    program: vec![],
                };

                lambda_block
                    .program
                    .push(LinearInstruction::AcceptToFormals {
                        static_formals_list: formals_vec_ref.clone(),
                    });

                // Make body
                let mut labmda_body = vec![];
                body.iter().for_each(|f| {
                    labmda_body.extend_from_slice(&self.expr_to_instructions(f.clone()))
                });

                let return_reg = self.make_reg_name();
                labmda_body.push(LinearInstruction::PopFromStack {
                    register: return_reg.clone(),
                });
                labmda_body.push(LinearInstruction::Return { value: return_reg });

                lambda_block.program.extend_from_slice(&labmda_body);

                self.lambda_map
                    .insert(anon_lambda_name.clone(), lambda_block);

                // Final thing return initialized fuction pointer
                let reg = self.make_reg_name();

                let initialized_func_pointer = FunctionPointer {
                    actual_func: anon_lambda_name,
                    formals_list: formals_vec_ref,
                };
                instr_buf.push(LinearInstruction::InitializeFunctionPointer {
                    function: initialized_func_pointer,
                    from_scope: Scope::Current,
                    outpu_reg: reg.clone(),
                });

                instr_buf.push(LinearInstruction::PushToStack { register: reg });
            }
            Expression::Cond(cases) => {
                // Conditions and Branches if true
                // Can internally just call and? or better just impl check here?
                // Shoul add an instruction for checking booleans somehow?
                // Can be done in cond instruction taking reg to check.
                for case in cases {
                    instr_buf.extend_from_slice(&self.expr_to_instructions(case.0));
                    let reg_to_check = self.make_reg_name();
                    instr_buf.push(LinearInstruction::PopFromStack {
                        register: reg_to_check.clone(),
                    });

                    instr_buf.push(LinearInstruction::Cond {
                        condition: reg_to_check,
                        branc_if_true: Branch {
                            program: self.expr_to_instructions(case.1),
                        },
                    });
                }
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
                    instr_buf.push(LinearInstruction::PopFromStack {
                        register: data_reg.clone(),
                    });

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
                for body_expr in body.iter().enumerate() {
                    instr_buf.extend_from_slice(&self.expr_to_instructions(body_expr.1.clone()));
                    instr_buf.push(LinearInstruction::PopFromStack {
                        register: body_res_reg.clone(),
                    });
                }
                instr_buf.push(LinearInstruction::PushToStack {
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
            Expression::Atom(_atom) => {
                // Idk is this even possible
                unimplemented!();
            }
            Expression::Identifier(_ident) => {
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
    fn make_anon_lambda_name(&mut self) -> String {
        let temp = "_".to_owned() + &self.anon_lambda_counter.to_string();
        self.anon_lambda_counter += 1;
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum StaticData {
    Bool(bool),
    Integer(i32),
    String(String),
    Identifier(String),
    List(Vec<StaticData>),
}

#[cfg(test)]
mod tests {
    use little_parser::Parser;

    use crate::{LinearBlock, LinearInstruction, Register, StaticData, StaticRef, Translator};

    #[test]
    fn it_works_init_1() {
        let mut parser = Parser::init_with_string(r#"('5)"#);
        let ast = parser.re_program();

        let mut translator = Translator::default();

        let incomplete_res = translator.ast_to_intermediate_representation(ast);

        println!("translator: {:#?}", translator);

        assert_eq!(
            incomplete_res,
            LinearBlock {
                ident: "main".into(),
                program: [
                    LinearInstruction::StaticRefToRegister {
                        static_ref: StaticRef {
                            refname: "static0".into(),
                            reftype: StaticData::Integer(5,),
                        },
                        to_reg: Register {
                            virtual_ident: "r0".into(),
                        },
                    },
                    LinearInstruction::PushToStack {
                        register: Register {
                            virtual_ident: "r0".into(),
                        },
                    },
                ]
                .to_vec(),
            }
        );
    }
}
