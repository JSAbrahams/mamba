⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

# 2.3 📦 Modules

## 📄 Contents

### [2.3.1 Script](script.md)
### [2.3.2 Class](class.md)
### [2.3.3 Types](types.md)
### [2.3.4 Type Aliases](type_alias.md)

## Introduction

A module is one of the following:

Module type | Description
------------|--------------
script      | A sequence of instructions.
type        | A blueprint for a class, defines the behaviour of a class.
stateful    | A blueprint for an instance of a stateful object. Defines the behaviour of that object, and may implement a type.
stateless   | A blueprint for a singleton, stateless object. Only one instance of such an object can exist, and its state may never change. It can implement a type.

A file may contain either a script accompanied by functions, or a mix of type, stateful and stateless modules. 
It is generally best to have at most one type per file for readability's sake.
