# Module

A module is one of the following:

Module   | Description
---------|--------------
Script   | A sequence of Instructions
Util     | A stateless collection of functions
Type     | A blueprint for a class, defines the behaviour of a class
Class    | A blueprint for an instance of an object. Defines the behaviour of that object and may implement a type

A file may contain either a script accompanied by functions, or a mix of types, utils and classes. Ideally, utils and
classes that belong together should be grouped together. Furthermore, it is generally best to have at most one type per
file for readability sake.
