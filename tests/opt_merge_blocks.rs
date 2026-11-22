use waffle::{Block, BlockTarget, FunctionBody, Operator, OptOptions, Terminator, Type};
use Type::I32;

fn block_target(block: Block) -> BlockTarget {
    BlockTarget {
        block,
        args: Vec::new(),
    }
}

fn make_br_br_ret(const_bb: usize, iadd_bb: usize) -> FunctionBody {
    let mut body = FunctionBody::default();
    let b0 = body.add_block();
    let b1 = body.add_block();
    let b2 = body.add_block();

    let v0 = body.add_blockparam(b0, I32);
    let bbs = [b0, b1, b2];
    let v1 = body.add_op(bbs[const_bb], Operator::I32Const { value: 42 }, &[], &[I32]);
    let v2 = body.add_op(bbs[iadd_bb], Operator::I32Add, &[v0, v1], &[I32]);

    body.entry = b0;
    body.set_terminator(
        b0,
        Terminator::Br {
            target: block_target(b1),
        },
    );
    body.set_terminator(
        b1,
        Terminator::Br {
            target: block_target(b2),
        },
    );
    body.set_terminator(b2, Terminator::Return { values: vec![v2] });
    body.recompute_edges();
    body.validate().unwrap();
    body
}

#[test]
fn br_br_ret_with_inst() {
    let mut body = make_br_br_ret(1, 2);
    expect_test::expect![[r#"
        function() ->  {
          block0(v0: i32): # 
            # preds: 
            # succs: block1 ()
            br block1()
          block1(): # 
            # preds: block0 ()
            # succs: block2 ()
            v1 = i32const<42>  # i32  
            br block2()
          block2(): # 
            # preds: block1 ()
            # succs: 
            v2 = i32add v0, v1 # i32  
            return v2
        }
    "#]]
    .assert_eq(&body.display("", None).to_string());

    body.optimize(&OptOptions::default());
    expect_test::expect![[r#"
        function() ->  {
          block0(v0: i32): # 
            # preds: 
            # succs: 
            v1 = i32const<42>  # i32  
            v2 = i32add v0, v1 # i32  
            return v2
          block1(): # 
            # preds: 
            # succs: 
            no_terminator
          block2(): # 
            # preds: 
            # succs: 
            no_terminator
        }
    "#]]
    .assert_eq(&body.display("", None).to_string());
}

#[test]
fn br_br_ret_no_inst() {
    let mut body = make_br_br_ret(2, 2);
    expect_test::expect![[r#"
        function() ->  {
          block0(v0: i32): # 
            # preds: 
            # succs: block1 ()
            br block1()
          block1(): # 
            # preds: block0 ()
            # succs: block2 ()
            br block2()
          block2(): # 
            # preds: block1 ()
            # succs: 
            v1 = i32const<42>  # i32  
            v2 = i32add v0, v1 # i32  
            return v2
        }
    "#]]
    .assert_eq(&body.display("", None).to_string());

    body.optimize(&OptOptions::default());
    expect_test::expect![[r#"
        function() ->  {
          block0(v0: i32): # 
            # preds: 
            # succs: 
            v1 = i32const<42>  # i32  
            v2 = i32add v0, v1 # i32  
            return v2
          block1(): # 
            # preds: 
            # succs: 
            no_terminator
          block2(): # 
            # preds: 
            # succs: 
            no_terminator
        }
    "#]]
    .assert_eq(&body.display("", None).to_string());
}

#[test]
fn condbr_br_ret() {
    let mut body = FunctionBody::default();
    let b0 = body.add_block();
    let b1 = body.add_block();
    let b2 = body.add_block();
    let b3 = body.add_block();

    let v0 = body.add_blockparam(b0, I32);
    let v1 = body.add_blockparam(b0, I32);

    body.entry = b0;
    body.set_terminator(
        b0,
        Terminator::CondBr {
            cond: v0,
            if_true: block_target(b1),
            if_false: block_target(b3),
        },
    );
    body.set_terminator(
        b1,
        Terminator::Br {
            target: block_target(b2),
        },
    );
    body.set_terminator(b2, Terminator::Return { values: vec![v0] });
    body.set_terminator(b3, Terminator::Return { values: vec![v1] });
    body.recompute_edges();
    body.validate().unwrap();

    expect_test::expect![[r#"
        function() ->  {
          block0(v0: i32, v1: i32): # 
            # preds: 
            # succs: block1 (), block3 ()
            if v0, block1(), block3()
          block1(): # 
            # preds: block0 ()
            # succs: block2 ()
            br block2()
          block2(): # 
            # preds: block1 ()
            # succs: 
            return v0
          block3(): # 
            # preds: block0 ()
            # succs: 
            return v1
        }
    "#]]
    .assert_eq(&body.display("", None).to_string());

    body.optimize(&OptOptions::default());
    expect_test::expect![[r#"
        function() ->  {
          block0(v0: i32, v1: i32): # 
            # preds: 
            # succs: block1 (), block3 ()
            if v0, block1(), block3()
          block1(): # 
            # preds: block0 ()
            # succs: 
            return v0
          block2(): # 
            # preds: 
            # succs: 
            no_terminator
          block3(): # 
            # preds: block0 ()
            # succs: 
            return v1
        }
    "#]]
    .assert_eq(&body.display("", None).to_string());
}

#[test]
fn merged_block_preserves_successor_edge_indices() {
    let mut body = FunctionBody::default();
    let entry = body.add_block();
    let branch = body.add_block();
    let left = body.add_block();
    let right = body.add_block();
    let cond = body.add_blockparam(entry, I32);
    body.entry = entry;
    body.set_terminator(
        entry,
        Terminator::Br {
            target: block_target(branch),
        },
    );
    body.set_terminator(
        branch,
        Terminator::CondBr {
            cond,
            if_true: block_target(left),
            if_false: block_target(right),
        },
    );
    body.set_terminator(left, Terminator::Return { values: vec![] });
    body.set_terminator(right, Terminator::Return { values: vec![] });
    body.recompute_edges();
    body.optimize(&OptOptions::default());
    body.validate().unwrap();
    for (block, def) in body.blocks.entries() {
        assert_eq!(def.succs.len(), def.pos_in_succ_pred.len());
        assert_eq!(def.preds.len(), def.pos_in_pred_succ.len());
        for (i, &succ) in def.succs.iter().enumerate() {
            let pred_index = def.pos_in_succ_pred[i];
            assert_eq!(body.blocks[succ].preds[pred_index], block);
            assert_eq!(body.blocks[succ].pos_in_pred_succ[pred_index], i);
        }
    }
}
