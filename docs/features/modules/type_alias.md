⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.3 📦 Modules](README.md)

# 2.3.4 Type Aliases

_Note_ Type aliases and the `type ... when ...` refinement described here are future work.
None of this page is implemented yet.
The traits, classes and error types it builds on are written in current syntax, the `type` declarations are not.

In certain situations, we want to make sure that certain methods can only be called when an instance of a class is in a certain state.
This can be achieved using type aliases and type refinement.

I have a trait `Matrix`:

    trait Matrix where
        def determinant(self) -> Float
        def is_invertible(self) -> Bool
        def solve(self, u: Float, v: Float) -> List[Float] ! MatrixErr
        def kernel_basis(self) -> List[Float] ! MatrixErr
    end

    class MatrixErr(msg: Str): Exception(msg)

And I define the following type aliases:

    type InvertibleMatrix2x2: Matrix2x2 when
        self.is_invertible() else MatrixErr("Matrix is singular.")

    type SingularMatrix2x2: Matrix2x2 when
        not self.is_invertible() else MatrixErr("Matrix is invertible.")

We can do the following:

    class Matrix2x2(a: Float, b: Float, c: Float, d: Float): Matrix where
        def determinant(self) -> Float := self.a * self.d - self.b * self.c

        def is_invertible(self) -> Bool := self.determinant() != 0.0

        # Solves this matrix against the vector (u, v) by Cramer's rule.
        def solve(self: InvertibleMatrix2x2, u: Float, v: Float) -> List[Float] ! MatrixErr := do
            def det := self.determinant()
            def x := u * self.d - self.b * v
            def y := self.a * v - u * self.c
            [x / det, y / det]
        end

        # A singular 2x2 matrix maps the whole plane onto a line, so its kernel
        # is spanned by a single non-zero vector.
        def kernel_basis(self: SingularMatrix2x2) -> List[Float] ! MatrixErr := do
            if self.a = 0.0 and self.b = 0.0 then [1.0, 0.0] else [self.b, -self.a]
        end
    end

Whether the matrix is invertible is now part of each method's signature.
`solve` can only be called on an `InvertibleMatrix2x2`, and `kernel_basis` only on a `SingularMatrix2x2`.
