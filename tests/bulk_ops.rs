//! Validate expanded bulk-memory operations against the indexed segment model.

use anyhow::Result;
use waffle::wasmparser::{Operator, Parser, Payload, Validator};
use waffle::{FrontendOptions, Module, OptOptions};

#[test]
fn expanded_bulk_operations_preserve_indices_and_validate() -> Result<()> {
    let bytes = wat::parse_str(
        r#"(module
        (memory 1)
        (table 2 externref)
        (data (i32.const 0) "active")
        (data $payload "passive")
        (data (i32.const 16) "another")
        (func (export "init")
            (memory.init $payload (i32.const 32) (i32.const 0) (i32.const 7))
            (data.drop $payload)
            (table.fill (i32.const 0) (ref.null extern) (i32.const 2))))"#,
    )?;
    let mut module = Module::from_wasm_bytes(&bytes, &FrontendOptions::default())?;
    module.expand_all_funcs()?;
    module.per_func_body(|func| func.optimize(&OptOptions::default()));
    let output = module.to_wasm_bytes()?;
    Validator::new().validate_all(&output)?;
    let mut operations = Vec::new();
    let mut data_count = None;
    for payload in Parser::new(0).parse_all(&output) {
        match payload? {
            Payload::DataCountSection { count, .. } => data_count = Some(count),
            Payload::CodeSectionEntry(body) => {
                for op in body.get_operators_reader()? {
                    match op? {
                        Operator::MemoryInit { data_index, mem } => {
                            operations.push((0, data_index, mem))
                        }
                        Operator::DataDrop { data_index } => operations.push((1, data_index, 0)),
                        Operator::TableFill { table } => operations.push((2, table, 0)),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    assert_eq!(data_count, Some(3));
    assert_eq!(operations, [(0, 1, 0), (1, 1, 0), (2, 0, 0)]);
    Ok(())
}
