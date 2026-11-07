<p align="center">
    <img src="../../../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Python backend

Converts a checked `ASTTy` to Python source in two steps.

- `convert/`: lowers `ASTTy` to `PythonCore`, a small tree close to Python.
  It keeps state while walking, since some constructs translate differently by context.
  It also collects the imports the output needs, such as `typing` names under `--annotate`.
- `ast/`: defines `PythonCore` and prints it as a `String`, near one-to-one plus indentation.

It errors on a construct it does not support yet, or on an `ASTTy` the checker should have rejected.
