# Real CPython module-level builtins, kept here purely so context-building has at least one
# typed and one untyped top-level (non-class) assignment to parse from a stub file:
#
# - `__debug__` is a real builtin bool, True unless Python is run with `-O`; code (and `assert`)
#   uses it to guard debug-only branches.
# - `__all__` is a real, extremely common module-level convention: a list of the names a module
#   exports via `from module import *`.
#
# NOTE: neither is actually usable from Mamba source yet. Top-level (non-class) fields are parsed
# into `Context.fields`, but nothing ever looks them up from there -- see "Top-level fields are
# parsed but never looked up" in tests/README.md. `print(__debug__)` in a .mamba file still fails
# with "Undefined variable: __debug__" even with this file present. They're kept anyway because
# they're genuine builtins (accurate documentation of what real Python provides) and their mere
# presence here already exercises the module-level Statement::Assignment / Statement::TypedAssignment
# parsing in check/context/python.rs for every test that runs check_all (i.e. no separate .mamba
# fixture is possible or needed for these two lines).
__debug__: bool = True
__all__ = []
