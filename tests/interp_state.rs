use anyhow::{bail, Context, Result};
use waffle::entity::EntityRef;
use waffle::{
    ConstVal, ExportKind, FrontendOptions, Func, InterpContext, InterpMemoryAccessKind, Module,
};

fn export_func(module: &Module<'_>, name: &str) -> Result<Func> {
    let export = module
        .exports
        .iter()
        .find(|export| export.name == name)
        .with_context(|| format!("missing export {name}"))?;
    match export.kind {
        ExportKind::Func(function) => Ok(function),
        _ => bail!("export {name} is not a function"),
    }
}

#[test]
fn interpreter_tracks_externref_table_size_and_null_growth() -> Result<()> {
    let bytes = wat::parse_str(
        r#"(module
            (table 3 10 externref)
            (func (export "size") (result i32)
                table.size)
            (func (export "grow") (param $delta i32) (result i32)
                ref.null extern
                local.get $delta
                table.grow))"#,
    )?;
    let mut module = Module::from_wasm_bytes(&bytes, &FrontendOptions::default())?;
    module.expand_all_funcs()?;
    let size = export_func(&module, "size")?;
    let grow = export_func(&module, "grow")?;
    let mut context = InterpContext::new(&module)?;

    assert_eq!(
        context.call(&module, size, &[]).ok()?.as_slice(),
        [ConstVal::I32(3)]
    );
    assert_eq!(
        context
            .call(&module, grow, &[ConstVal::I32(4)])
            .ok()?
            .as_slice(),
        [ConstVal::I32(3)]
    );
    assert_eq!(
        context.call(&module, size, &[]).ok()?.as_slice(),
        [ConstVal::I32(7)]
    );
    assert_eq!(
        context
            .call(&module, grow, &[ConstVal::I32(4)])
            .ok()?
            .as_slice(),
        [ConstVal::I32(u32::MAX)]
    );
    assert_eq!(
        context.call(&module, size, &[]).ok()?.as_slice(),
        [ConstVal::I32(7)]
    );
    Ok(())
}

#[test]
fn interpreter_records_exact_store_and_growth_ranges() -> Result<()> {
    let bytes = wat::parse_str(
        r#"(module
            (memory 1 3)
            (func (export "write") (param $base i32)
                local.get $base
                i32.const 287454020
                i32.store offset=4
                local.get $base
                i32.load offset=4
                drop
                i32.const 1
                memory.grow
                drop))"#,
    )?;
    let mut module = Module::from_wasm_bytes(&bytes, &FrontendOptions::default())?;
    module.expand_all_funcs()?;
    let write = export_func(&module, "write")?;
    let mut context = InterpContext::new(&module)?;

    context.call(&module, write, &[ConstVal::I32(8)]).ok()?;
    let accesses = context.take_memory_accesses();
    assert_eq!(accesses.len(), 3);
    assert_eq!(
        (
            accesses[0].kind,
            accesses[0].memory.index(),
            accesses[0].addr,
            accesses[0].len
        ),
        (InterpMemoryAccessKind::Write, 0, 12, 4)
    );
    assert_eq!(
        (
            accesses[1].kind,
            accesses[1].memory.index(),
            accesses[1].addr,
            accesses[1].len
        ),
        (InterpMemoryAccessKind::Read, 0, 12, 4)
    );
    assert_eq!(
        (
            accesses[2].kind,
            accesses[2].memory.index(),
            accesses[2].addr,
            accesses[2].len
        ),
        (InterpMemoryAccessKind::Write, 0, 65_536, 65_536)
    );
    assert!(context.take_memory_accesses().is_empty());
    Ok(())
}

#[test]
fn interpreter_i32_load8_u_accepts_last_memory_byte() -> Result<()> {
    let bytes = wat::parse_str(
        r#"(module
            (memory 1)
            (func (export "read") (result i32)
                i32.const 65535
                i32.load8_u))"#,
    )?;
    let mut module = Module::from_wasm_bytes(&bytes, &FrontendOptions::default())?;
    module.expand_all_funcs()?;
    let read = export_func(&module, "read")?;
    let mut context = InterpContext::new(&module)?;
    assert_eq!(
        context.call(&module, read, &[]).ok()?.as_slice(),
        [ConstVal::I32(0)]
    );
    let accesses = context.take_memory_accesses();
    assert_eq!(accesses.len(), 1);
    assert_eq!(
        (
            accesses[0].kind,
            accesses[0].memory.index(),
            accesses[0].addr,
            accesses[0].len
        ),
        (InterpMemoryAccessKind::Read, 0, 65_535, 1)
    );
    Ok(())
}
