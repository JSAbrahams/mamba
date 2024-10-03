⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

# 2.4 ⛑ Safety

## 📄 Contents

### [2.4.1 Types](types.md)
### [2.4.2 Error Handling](error_handling.md)
### [2.4.3 Null Safety](null_safety.md)
### [2.4.4 Generics](generics.md)

## Introduction

There are several safety features baked into the language to make it easier to catch bugs early on.

We have a type system so we can define functions which may only manipulate a certain type of object.
We also have type refinement features to further enhance function definitions.
We can also specify what type of object they return.
To extend this we have also added generics to the language, which can be useful when for instance defining collections of objects.

We have certain error handling mechanisms built in, which encourage error handling on site help to make it clear visually where errors might occur.
To avoid undefined behaviour, we also baked null safety features into the language.
