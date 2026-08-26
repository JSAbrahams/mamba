pub mod cranelift;
pub mod python;

/// Which backend the pipeline should target.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Backend {
    /// Transpile to Python source (the original, default behavior).
    #[default]
    Python,
    /// Compile and link a native executable via the Cranelift backend.
    ///
    /// `target`, if given, is a target triple (e.g. `x86_64-unknown-linux-gnu`) passed on to
    /// Cranelift; if `None`, the host triple is used.
    Bin { target: Option<String> },
    /// Compile via the Cranelift backend and print the resulting disassembly to stdout, instead
    /// of linking an executable. No file is written -- pipe stdout if you want to save it.
    /// Printed in AT&T syntax -- Cranelift's own disassembler doesn't support switching to Intel
    /// syntax.
    ///
    /// Only shows instructions, not the data section -- a string literal is compiled into the
    /// object's data section, not the instruction stream, so it won't appear in this output at
    /// all; the instructions will only show it being loaded by an opaque symbol name.
    ///
    /// `target`, if given, is a target triple (e.g. `x86_64-unknown-linux-gnu`) passed on to
    /// Cranelift; if `None`, the host triple is used.
    Asm { target: Option<String> },
}
