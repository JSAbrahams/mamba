⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.3 📦 Modules](README.md)

# 2.3.3 Type

A trait describes the properties a value of that type should have.
Each such property is a definition.
A definition is then either a method or a variable.
Only classes can implement traits.

Say I have a vector, I could define the trait as follows:

    trait Vector where
        def norm(self) -> Float
        def last_scaled_by(self) -> Float?
        def scale(mut self, factor: Float) -> Bool ! VectorErr
        def normalise(mut self) -> Bool ! VectorErr
    end

    class VectorErr(msg: Str): Exception(msg)

This is akin to an abstract base class in Python, but more compact.
A definition in a trait is a signature with no body.
Note that each method takes an explicit `self` argument, and that `self` means the method may not change the state of the instance.

Now any class that implements `Vector` must have these definitions.

    class Vector2(mut x: Float, mut y: Float): Vector where
        def mut last_factor: Float? := None

        def norm(self) -> Float := (self.x * self.x + self.y * self.y) ^ 0.5

        def last_scaled_by(self) -> Float? := self.last_factor

        def scale(mut self, factor: Float) -> Bool ! VectorErr := do
            if factor = 0.0 then ! VectorErr("Cannot scale a vector by zero.")
            self.x := self.x * factor
            self.y := self.y * factor
            self.last_factor := factor
            True
        end

        def normalise(mut self) -> Bool ! VectorErr := do
            def length := self.norm()
            if length = 0.0 then ! VectorErr("Cannot normalise the zero vector.")
            self.scale(1.0 / length)
        end
    end

The constructor arguments `x` and `y` are fields, and are reached as `self.x` and `self.y` inside the body.
A class may name more than one parent, as in `class Vector2(...): Vector, Measurable where ... end`.
