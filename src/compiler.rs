use inkwell::{
    IntPredicate,
    builder::Builder,
    context::Context,
    module::Module,
    types::IntType,
    values::{FunctionValue, PointerValue},
};

use crate::shared::Operation;

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    i8_type: IntType<'ctx>,
    i32_type: IntType<'ctx>,
    i64_type: IntType<'ctx>,
    putchar_fn: FunctionValue<'ctx>,
    getchar_fn: FunctionValue<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("program");
        let builder = context.create_builder();

        let i8_type = context.i8_type();
        let i32_type = context.i32_type();
        let i64_type = context.i64_type();

        let putchar_type = i32_type.fn_type(&[i8_type.into()], false);
        let putchar_fn = module.add_function("putchar", putchar_type, None);

        let getchar_type = i32_type.fn_type(&[], false);
        let getchar_fn = module.add_function("getchar", getchar_type, None);

        CodeGen {
            context,
            module,
            builder,
            i8_type,
            i32_type,
            i64_type,
            putchar_fn,
            getchar_fn,
        }
    }

    pub fn compile(&self, operations: &[Operation]) -> FunctionValue<'ctx> {
        let main_type = self.i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_type, None);

        let entry_bb = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry_bb);

        let array_type = self.i8_type.array_type(30000);
        let memory = self.builder.build_alloca(array_type, "memory").unwrap();
        self.builder
            .build_memset(
                memory,
                16,
                self.i8_type.const_zero(),
                self.i64_type.const_int(30000, false),
            )
            .unwrap();

        let data_ptr = self
            .builder
            .build_alloca(self.i64_type, "data_ptr")
            .unwrap();
        self.builder
            .build_store(data_ptr, self.i64_type.const_zero())
            .unwrap();

        let ret_block = self.context.append_basic_block(main_fn, "ret");

        if operations.is_empty() {
            self.builder.build_unconditional_branch(ret_block).unwrap();
        } else {
            let mut op_blocks = Vec::with_capacity(operations.len());
            for i in 0..operations.len() {
                let bb = self
                    .context
                    .append_basic_block(main_fn, &format!("op_{}", i));
                op_blocks.push(bb);
            }

            self.builder
                .build_unconditional_branch(op_blocks[0])
                .unwrap();

            for (i, op) in operations.iter().enumerate() {
                self.builder.position_at_end(op_blocks[i]);

                let next_block = if i + 1 < operations.len() {
                    op_blocks[i + 1]
                } else {
                    ret_block
                };

                match op {
                    Operation::Next => {
                        let dp = self
                            .builder
                            .build_load(self.i64_type, data_ptr, "dp")
                            .unwrap()
                            .into_int_value();
                        let inc = self
                            .builder
                            .build_int_add(dp, self.i64_type.const_int(1, false), "dp_inc")
                            .unwrap();
                        self.builder.build_store(data_ptr, inc).unwrap();
                        self.builder.build_unconditional_branch(next_block).unwrap();
                    }
                    Operation::Prev => {
                        let dp = self
                            .builder
                            .build_load(self.i64_type, data_ptr, "dp")
                            .unwrap()
                            .into_int_value();
                        let dec = self
                            .builder
                            .build_int_sub(dp, self.i64_type.const_int(1, false), "dp_dec")
                            .unwrap();
                        self.builder.build_store(data_ptr, dec).unwrap();
                        self.builder.build_unconditional_branch(next_block).unwrap();
                    }
                    Operation::Increment => {
                        let cell = self.build_cell_ptr(memory, data_ptr);
                        let val = self
                            .builder
                            .build_load(self.i8_type, cell, "val")
                            .unwrap()
                            .into_int_value();
                        let inc = self
                            .builder
                            .build_int_add(val, self.i8_type.const_int(1, false), "inc")
                            .unwrap();
                        self.builder.build_store(cell, inc).unwrap();
                        self.builder.build_unconditional_branch(next_block).unwrap();
                    }
                    Operation::Decrement => {
                        let cell = self.build_cell_ptr(memory, data_ptr);
                        let val = self
                            .builder
                            .build_load(self.i8_type, cell, "val")
                            .unwrap()
                            .into_int_value();
                        let dec = self
                            .builder
                            .build_int_sub(val, self.i8_type.const_int(1, false), "dec")
                            .unwrap();
                        self.builder.build_store(cell, dec).unwrap();
                        self.builder.build_unconditional_branch(next_block).unwrap();
                    }
                    Operation::Output => {
                        let cell = self.build_cell_ptr(memory, data_ptr);
                        let val = self.builder.build_load(self.i8_type, cell, "val").unwrap();
                        self.builder
                            .build_call(self.putchar_fn, &[val.into()], "putchar")
                            .unwrap();
                        self.builder.build_unconditional_branch(next_block).unwrap();
                    }
                    Operation::Input => {
                        let cell = self.build_cell_ptr(memory, data_ptr);
                        let call = self
                            .builder
                            .build_call(self.getchar_fn, &[], "getchar")
                            .unwrap();
                        let val = call.try_as_basic_value().basic().unwrap().into_int_value();
                        let trunc = self
                            .builder
                            .build_int_truncate(val, self.i8_type, "char")
                            .unwrap();
                        self.builder.build_store(cell, trunc).unwrap();
                        self.builder.build_unconditional_branch(next_block).unwrap();
                    }
                    Operation::JumpIfZero(target) => {
                        let cell = self.build_cell_ptr(memory, data_ptr);
                        let val = self
                            .builder
                            .build_load(self.i8_type, cell, "val")
                            .unwrap()
                            .into_int_value();
                        let is_zero = self
                            .builder
                            .build_int_compare(
                                IntPredicate::EQ,
                                val,
                                self.i8_type.const_zero(),
                                "iszero",
                            )
                            .unwrap();
                        self.builder
                            .build_conditional_branch(is_zero, op_blocks[*target], next_block)
                            .unwrap();
                    }
                    Operation::JumpIfNonzero(target) => {
                        let cell = self.build_cell_ptr(memory, data_ptr);
                        let val = self
                            .builder
                            .build_load(self.i8_type, cell, "val")
                            .unwrap()
                            .into_int_value();
                        let is_nz = self
                            .builder
                            .build_int_compare(
                                IntPredicate::NE,
                                val,
                                self.i8_type.const_zero(),
                                "isnz",
                            )
                            .unwrap();
                        self.builder
                            .build_conditional_branch(is_nz, op_blocks[*target], next_block)
                            .unwrap();
                    }
                }
            }
        }

        self.builder.position_at_end(ret_block);
        self.builder
            .build_return(Some(&self.i32_type.const_int(0, false)))
            .unwrap();

        main_fn
    }

    fn build_cell_ptr(
        &self,
        memory: PointerValue<'ctx>,
        data_ptr_ptr: PointerValue<'ctx>,
    ) -> PointerValue<'ctx> {
        let data = self
            .builder
            .build_load(self.i64_type, data_ptr_ptr, "data")
            .unwrap()
            .into_int_value();
        let array_type = self.i8_type.array_type(30000);
        let indices = &[self.i64_type.const_zero(), data];
        unsafe {
            self.builder
                .build_in_bounds_gep(array_type, memory, indices, "cell")
        }
        .unwrap()
    }

    pub fn jit_run(self) {
        let engine = self
            .module
            .create_jit_execution_engine(inkwell::OptimizationLevel::None)
            .unwrap();
        unsafe {
            let address = engine.get_function_address("main").unwrap();
            let func: fn() -> i32 = std::mem::transmute(address);
            func();
        }
    }

    pub fn print_ir(&self) {
        println!("{}", self.module.print_to_string().to_string());
    }
}
