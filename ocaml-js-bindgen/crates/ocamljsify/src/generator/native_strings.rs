//! Optional engine-intrinsic byte-string copier, sharing the Rust instance's memory.
use wasm_encoder::*;

pub(super) fn copier() -> Vec<u8> {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types
        .ty()
        .function([ValType::EXTERNREF, ValType::I32], [ValType::I32]);
    types.ty().function([ValType::EXTERNREF], [ValType::I32]);
    types.ty().function([ValType::EXTERNREF, ValType::I32], []);
    module.section(&types);
    let mut imports = ImportSection::new();
    imports.import("wasm:js-string", "charCodeAt", EntityType::Function(0));
    imports.import("wasm:js-string", "length", EntityType::Function(1));
    imports.import(
        "env",
        "memory",
        EntityType::Memory(MemoryType {
            minimum: 0,
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        }),
    );
    module.section(&imports);
    let mut functions = FunctionSection::new();
    functions.function(2);
    module.section(&functions);
    let mut exports = ExportSection::new();
    exports.export("copy", ExportKind::Func, 2);
    module.section(&exports);
    let mut function = Function::new([(2, ValType::I32)]);
    for op in [
        Instruction::LocalGet(0),
        Instruction::Call(1),
        Instruction::LocalSet(2),
        Instruction::Block(BlockType::Empty),
        Instruction::Loop(BlockType::Empty),
        Instruction::LocalGet(3),
        Instruction::LocalGet(2),
        Instruction::I32GeU,
        Instruction::BrIf(1),
        Instruction::LocalGet(1),
        Instruction::LocalGet(3),
        Instruction::I32Add,
        Instruction::LocalGet(0),
        Instruction::LocalGet(3),
        Instruction::Call(0),
        Instruction::I32Store8(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }),
        Instruction::LocalGet(3),
        Instruction::I32Const(1),
        Instruction::I32Add,
        Instruction::LocalSet(3),
        Instruction::Br(0),
        Instruction::End,
        Instruction::End,
        Instruction::End,
    ] {
        function.instruction(&op);
    }
    let mut code = CodeSection::new();
    code.function(&function);
    module.section(&code);
    module.finish()
}

/// Optional packed-array construction avoids JS widening loops for native byte
/// output. The returned host string owns its characters independently of Wasm.
pub(super) fn reader() -> Vec<u8> {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().array(&StorageType::I16, true);
    let array = ValType::Ref(RefType {
        nullable: true,
        heap_type: HeapType::Concrete(0),
    });
    let string = ValType::Ref(RefType {
        nullable: false,
        heap_type: HeapType::Abstract {
            shared: false,
            ty: AbstractHeapType::Extern,
        },
    });
    types
        .ty()
        .function([array, ValType::I32, ValType::I32], [string]);
    types
        .ty()
        .function([ValType::I32, ValType::I32], [ValType::EXTERNREF]);
    module.section(&types);
    let mut imports = ImportSection::new();
    imports.import(
        "wasm:js-string",
        "fromCharCodeArray",
        EntityType::Function(1),
    );
    imports.import(
        "env",
        "memory",
        EntityType::Memory(MemoryType {
            minimum: 0,
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        }),
    );
    module.section(&imports);
    let mut functions = FunctionSection::new();
    functions.function(2);
    module.section(&functions);
    let mut exports = ExportSection::new();
    exports.export("read", ExportKind::Func, 1);
    module.section(&exports);
    let mut function = Function::new([(1, ValType::I32), (1, array)]);
    for op in [
        Instruction::LocalGet(1),
        Instruction::ArrayNewDefault(0),
        Instruction::LocalSet(3),
        Instruction::Block(BlockType::Empty),
        Instruction::Loop(BlockType::Empty),
        Instruction::LocalGet(2),
        Instruction::LocalGet(1),
        Instruction::I32GeU,
        Instruction::BrIf(1),
        Instruction::LocalGet(3),
        Instruction::LocalGet(2),
        Instruction::LocalGet(0),
        Instruction::LocalGet(2),
        Instruction::I32Add,
        Instruction::I32Load8U(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }),
        Instruction::ArraySet(0),
        Instruction::LocalGet(2),
        Instruction::I32Const(1),
        Instruction::I32Add,
        Instruction::LocalSet(2),
        Instruction::Br(0),
        Instruction::End,
        Instruction::End,
        Instruction::LocalGet(3),
        Instruction::I32Const(0),
        Instruction::LocalGet(1),
        Instruction::Call(0),
        Instruction::End,
    ] {
        function.instruction(&op);
    }
    let mut code = CodeSection::new();
    code.function(&function);
    module.section(&code);
    module.finish()
}
