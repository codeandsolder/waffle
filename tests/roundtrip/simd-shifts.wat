(module
  (func (export "shift16") (param v128 i32) (result v128)
    local.get 0
    local.get 1
    i16x8.shr_s)
  (func (export "shift32") (param v128 i32) (result v128)
    local.get 0
    local.get 1
    i32x4.shl)
  (func (export "shift64") (param v128 i32) (result v128)
    local.get 0
    local.get 1
    i64x2.shr_u))
