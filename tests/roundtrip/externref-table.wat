(module
  (table 8 externref)
  (func (export "clear")
    i32.const 0
    ref.null extern
    i32.const 8
    table.fill 0)
  (func (export "grow") (result i32)
    ref.null extern
    i32.const 1
    table.grow 0))
