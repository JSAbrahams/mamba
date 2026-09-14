⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.1 🔀 Control Flow](README.md)

# 2.1.2 Control Flow Statements

## While Statements

A while is useful when we want to execute a statement or expression, or block, multiple times as long as a certain condition holds.

A `while` statement has the form:

    while <expression> do <expression or statement> end

Unlike an `if`, the body of a `while` is always a block, so the `do` and `end` are not optional.

For instance

    while some_condition(a) do
        print("body of the loop.")
        self.some_function()
        print("end of the loop.")
    end

A `while` can never be used as an expression, as it does not evaluate to anything.

## Foreach Statements

A `for` loop is useful when we want to iterate over items in a collection.
As with a `while`, its body is always a block:

    for <identifier> in <expression> do <expression or statement> end

We can for instance do the following:

    for i in 0 .. 10 do print(i) end

Which prints numbers 0 till 9.
If we also want to print 10, we use the inclusive `..=` range operator:

    for i in 0 ..= 10 do print(i) end

We can also iterate over a set.

    def my_set := { "first", "second", "third", "last" }
    for item in my_set do print(item) end

A `for` can never be used as an expression, as it does not evaluate to anything.
