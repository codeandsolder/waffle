use anyhow::{bail, Context, Result};
use waffle::entity::EntityRef;
use waffle::{ConstVal, ExportKind, FrontendOptions, Func, InterpContext, Module};

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
                i32.const 1
                memory.grow
                drop))"#,
    )?;
    let mut module = Module::from_wasm_bytes(&bytes, &FrontendOptions::default())?;
    module.expand_all_funcs()?;
    let write = export_func(&module, "write")?;
    let mut context = InterpContext::new(&module)?;

    context.call(&module, write, &[ConstVal::I32(8)]).ok()?;
    let writes = context.take_memory_writes();
    assert_eq!(writes.len(), 2);
    assert_eq!(
        (writes[0].memory.index(), writes[0].addr, writes[0].len),
        (0, 12, 4)
    );
    assert_eq!(
        (writes[1].memory.index(), writes[1].addr, writes[1].len),
        (0, 65_536, 65_536)
    );
    assert!(context.take_memory_writes().is_empty());
    Ok(())
}
