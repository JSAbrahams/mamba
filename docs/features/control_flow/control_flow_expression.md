⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.1 🔀 Control Flow](README.md)

# 2.1.1 Control Flow Expressions

### If Expressions (or Statements)

If expressions are perhaps one of the most well known and universal programming constructs.

An if has either the form:
```
if <expression> then <expression or statement>
```
or:
```
if <expression> then <expression or statement> else <expression or statement>
```

For instance:
```
if b = 3 then print "b is three." else print "b is not three."
```

An `if` is an expression if:
-	Have an `else` branch.
-	Return an `<expression>` (not a `<statement>`) in both branches.
  An expression evaluates to a value, for instance, `10 * x` is an expression if `x` is a number for instance, whereas `print "hello world"` is a statement, as it does not return anything.

So, an `if` _expression_ has the form
```
if <expression> then <expression> else <expression>
```

An examplew would be:
```
def my_value = if a > 0 then 2E30 else 8E21
```

### Match Expressions (or Statements)

A `match` can be used match based on the value of an expression.
We can even match based on the type of the returned expression.

A `match` has the form:
```
match <expression> with { <expression> => <expression or statement> }
```

An example would be (if `b` is a number):
```
match b with
	1 => print "one"
	4 => print "four"
	5 => print "five"
```

We can also add a default case if:
```
match b with
	1 => print "one"
	4 => print "four"
	5 => print "five"
	_ => print "this is executed if we didn't match with any other"
```

A `match` is an expression if:
-	Every `match` arm returns an `<expression>` (not a `<statement>`).
	An expression must evaluate to a value, whereas a statement doesn't.
-	There is a match arm for every situation.
	This can be achieved by either exhaustively covering every possible value, or by having a default arm.

So, a `match` _expression_ has the form:
```
match <expression> with { <expression> => <expression> }
```
With the additional requirement that we have a arm for every possible value of a given input type.
