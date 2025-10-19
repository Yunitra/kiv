//! Textual backend that serializes MIR into a stable, human-readable form.

use kivc_mir::{MirFunction, MirInstr, MirProgram, MirTerminator};

pub fn emit_textual_ir(program: &MirProgram) -> String {
    let mut out = String::new();
    out.push_str("; Kiv textual IR (no LLVM)\n");
    for func in &program.functions {
        emit_function(&mut out, func);
    }
    out
}

fn emit_function(out: &mut String, func: &MirFunction) {
    out.push_str(&format!("func {}(params: {:?})\n", func.name, func.params));
    for block in &func.blocks {
        out.push_str(&format!("  block {}:\n", block.id.as_usize()));
        for instr in &block.instructions {
            match instr {
                MirInstr::Assign { dest, source } => {
                    out.push_str(&format!("    %{} = {:?}\n", dest.as_usize(), source));
                }
                MirInstr::BinOp {
                    dest,
                    op,
                    left,
                    right,
                } => {
                    out.push_str(&format!(
                        "    %{} = ({:?} {:?} {:?})\n",
                        dest.as_usize(),
                        op,
                        left,
                        right
                    ));
                }
                MirInstr::Call { dest, func, args } => {
                    if let Some(d) = dest {
                        out.push_str(&format!(
                            "    %{} = call {}({:?})\n",
                            d.as_usize(),
                            func,
                            args
                        ));
                    } else {
                        out.push_str(&format!("    call {}({:?})\n", func, args));
                    }
                }
                MirInstr::Nop => out.push_str("    nop\n"),
            }
        }
        match &block.terminator {
            MirTerminator::Return { value } => match value {
                Some(v) => out.push_str(&format!("    ret {:?}\n", v)),
                None => out.push_str("    ret\n"),
            },
            MirTerminator::Jump { target } => {
                out.push_str(&format!("    jmp {}\n", target.as_usize()));
            }
            MirTerminator::Branch {
                condition,
                true_block,
                false_block,
            } => {
                out.push_str(&format!(
                    "    br {:?}, {}, {}\n",
                    condition,
                    true_block.as_usize(),
                    false_block.as_usize()
                ));
            }
            MirTerminator::Unreachable => out.push_str("    unreachable\n"),
        }
    }
}
