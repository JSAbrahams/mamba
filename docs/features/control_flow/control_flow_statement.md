⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.1 🔀 Control Flow](README.md)

# 2.1.2 Control Flow Statements

### While Statements

A while is useful when we want to execute a statement or expression, or block, multiple times as long as a certain condition holds.

A `while` statement has the form:
```
while <expression> do <expression or statement>
```

For instance
```
while some_condition(a) do
	print "body of the loop."
	self.some_function()
	print "end of the loop."
```

A `while` can never be used as an expression, as it does not evaluate to anything.

### Foreach Statements

A `for` loop is useful when we want to iterate over items in a collection.

We can also do the following:
```
for i in 0..10 do print i
```
Which prints numbers 0 till 9.
If we also want to print 10, we use the inclusive `..=` range operator:
```
for i in 0..=10 do print i
```

We can also iterate over a set.
```
def my_set = { "first", "second", "third", "last" }
for item in my_set do print item
```

A `for` can never be used as an expression, as it does not evaluate to anything.
