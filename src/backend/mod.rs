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
    /// `target`, if given, is a target triple (e.g. `x86_64-unknown-linux-gnu`) passed on to
    /// Cranelift; if `None`, the host triple is used.
    Asm { target: Option<String> },
}
