(module
  (memory 1)
  (data $payload "hello")
  (func (export "init")
    i32.const 0
    i32.const 0
    i32.const 5
    memory.init $payload
    data.drop $payload))
