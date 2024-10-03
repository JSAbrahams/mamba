⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.4 ⛑ Safety](README.md)

# 2.4.1 Types

Often a distinction is made between static and dynamic typing. 

If the application were dynamically typed, we would constantly have to verify that variables are indeed what they claim to be.
Say I have a variable `beethoven`, can I assume that it is an instance of `Composer`?
   
    # how can I be sure that this function argument is a composer?
    def my_function(composer) => composer.composer_method()
    
There are of course ways to check this, such as using the `isinstance` method in Python, but this is rather tiresome.
To this end, we use types. 
A user defines a type `Composer`, which defines the behaviour of a composer:

    class Composer
        def composer_method(): Int => 10
    
And then we define the `my_function` as such:

    def my_function(composer: Composer) -> Int => composer.composer_method()
    
Now, in the body of the function, we can rest easy knowing that the passed variable is indeed a composer. 
It is actually now impossible to pass another variable type to the function, as this is statically checked by the type checker.
If it sees that we try to pass something that is not a composer, will give an error, meaning that the program wil not run.

In some programming languages, we have to explicitly state the type of each variable. 
This however makes the application rather verbose. 
Take for instance:

    def x: Int := 10                     # x is obviously an integer
    def c: Complex := Complex(10, 20)    # from the right hand side it is already clear that c is complex

Instead, we can use type inference. 
The type of every variable is inferred from the context in which it is used.

```
    def x := 10                 # x has type Int, we know this because 10 is an Int
    def c := Complex(10, 20)    # c has type Complex
    def y := 20.1               # 20.1 uses decimal notation, so we know y is a real number, or Real
    
    def z: Float := 10.5         # In some situations however, you still might want to explicitly mention the type
```

The program is still statically typed, but now we don't require the developer to write everything out in full.

### Type Aliases and Type Refinement

We can also use type aliases and type refinement to further refine types by adding conditions to them.
Say we have the following:

    type DeadComposer: Composer when
        self.death_date /= None else "Composer is not dead."

We now rewrite my_function so it only works for `DeadComposer`s:

    def my_function(composer: DeadComposer): Int => today.year - composer.death.year
    
Again, we can rest assured that `composer` is a `DeadComposer` in the body of the function. 
To use such a function, we must explicitly cast a `Composer`:

```
    def chopin := Composer("Chopin")
    
    if chopin isa DeadComposer then
        def years_ago := my_function(chopin)                    # chopin is dynamically casted to a DeadComposer
        print("{chopin.name} died {years_ago} years ago.")
```

This draws on concepts of **Design by Contract** philosophy.

Furthermore, it also allows us to explicitly define the state of an object, something which is often left ambiguous.
For instance, we can say a server is connected or disconnected by doing the following:

```
    type Server
        def private connected: Boolean
        def send_message(self: ConnectedServer, String) -> String

    type ConnectedServer isa Server when
        self.connected else "Server is not connected"
```

And we may then elsewhere implement this `Server` interface:

    class MyServer: Server
        def private connected := false
        
        def init() => ...
        
        def connect(ip: IpAddress) => ...
        
        # You can only call this function if I am a connected server
        def send_message(self: ConnectedServer, message: String) -> String => ...

This is a rather trivial example, but it shows how we can explicitly name the different states of a server.

## Type aliases

In some cases, for readability we might want to write a type alias. Say we have the following function:

    def distance_remaning(covered: Int) -> Int => self total - covered
    
The above seems simple, but there are two issues:
* At a glance, we cannot know what covered symbolises. Kilometers, meters? We can of course rename the variable, but
  in certain situations this makes the code rather verbose.
* We do no bounds checking here. What if covered is more than the total, or negative? We could add these bounds checks
  to the method. However, this makes the method more verbose. Ideally, we want to method to express in a concise manner 
  what it does without having a majority of the method being error handling code.
  
To solve the above two issues, we can use type aliases. Observe the following:

    type Kilometer: Int
    
Type `Kilometer` can do everything an `Int` can (we can use all the same operators), but using such an alias allows us
to more clearly express our ideas in the codebase without relying on documentation. (This is a recurring theme, source
code ideally should speak for itself without relying heavily on documentation.) We can rewrite the function as so:

    def distance_remaning(self, covered: Kilometer) -> Kilometer -> self.total - covered

## Type Refinement

Type refinement expands upon type aliases by defining certain conditions an object must adhere to to be considered that
type. This can also be used to enforce pre-conditions of a function or method when used as a parameter, and 
post-conditions when used as the return type. This is akin to the philosophy of Design by Contract.

Say we have a function:

    def f (x: Int) -> Int => 
        println("this number is even: {x}")
        x
    
In some situations, this function does not behave as we expect it to. It may print an uneven number. In such a
situation, we often turn to the design by contract philosophy, where a function has pre and post-conditions. There are
several traditional approaches to solving this problem, both if which are valid, though the preferred approach does 
depend on context:

Just return `x` if it is uneven and don't print anything using a simple `if`:

    def f (x: Int) -> Int => 
        if x mod 2 /= 0 return x
        print("this number is even: {x}")
        x
    
Raise and error if `x` is uneven:
    
    def f (x: Int) -> Int raise Err => 
        if x mod 2 /= 0 raise Err("Expected x to be even.")
        print("this number is even: {x}")
        x
    
However, in the above code, we see that writing pre-conditions can get out of hand, and we might want to use the same 
pre-conditions for multiple functions, which results in duplicate code. This is a situation where type aliases with 
conditions can come in handy. Type aliases encourages decentralisation. The logic of a type is closely linked to the
type itself, instead of having to manually check the a type adheres to certain conditions every time it is used.

We can use a trivial type `EvenNum` to demonstrate how one would use conditions in a type alias. Say we define the
type-alias `EvenNum`:

    type EvenNum: Int when
        self mod 2 = 0 # we can list more conditions below this one. They must all evaluate to a boolean.
        
Which is the same as:
   
    type EvenNum: Int when
        self mod 2 = 0
        
We may also chooose to add a descriptive error message:

    type EvenNum: Int when
        self mod 2 = 0 else "{self} is an uneven number"

This defines all `Int`, or Integers, that are even. That is, the condition listed above holds. This is similar to creating
a new class `EvenNum` which is a `Int`, and verifying that these properties hold.

We can now redefine the function as follows:

    # EvenNum has conditions, so we need to state that it may raise an error
    def g (x: EvenNum) -> Int raise Err => 
        print("this number is even: {x}")
        x
    
Now, the actual type of the argument describes what conditions the argument adheres to, instead of having to manually
check these in the body of the function. We now know that these conditions hold in the body of the function. We can cast
any variable that is an `Int` to `EvenNum`. During casting, the defined conditions are checked, and the respective error
is thrown if a condition does not hold:

```
    # We can cast x to an EvenNum, which might give an error
    def x := random_int() # here x is an Int
    def y := x as EvenNum
    def first := g(y)
    
    # We can also pass it immediately if we want, in which case it is casted to EvenNum
    def z := random_int()
    def second := g(z)
    # which is the same as
    def second_with_case := g(z as EvenNum)
    
    # Or just say that the variable is an EvenNum upon instantiation
    def y: EvenNum := random_int()
    def third := g(y)
    
    # We can also use the isa to check that the conditions holds without raising an error
    def a := random_int()
    # notice how we don't have to cast a to an EvenNum if the condition holds.
    # We know that the then branch of the if is only executed if a is an EvenNum, so we assign it the type EvenNum
    fourth := if a isa EvenNum then g(a) else 0
    
    # If it can be statically verified that the propties hold, it is not necessary to handle any type specific errors
    def c := 2
    def fifth := g(c)
    
    # first, second, third, fourth, and fifth all have type Int
```

We can also use it as a sort of post-condition of the function. We ensure that the function returns an `EvenNum`:

    def g (x: EvenNum): EvenNum raise Err => 
        print("this number is even: {x}")
        def y := x + some_other_function(x)
        y as EvenNum
        
Or:

    def g (x: EvenNum): EvenNum raise Err => 
        print("this number is even: [x]")
        def y: EvenNum := x + some_other_function(x)
        y

We can even ensure that the function never returns an error:

    def h (x: EvenNum): EvenNum => 
        print("this number is even: {x}")
        def y := x + some_other_function(x)
        if y isa EvenNum then
            y # type sensitive flow ensure that this is an EvenNum
        else x

So now:

```
    def x := 10  # here x is an Int
    def a := g(x as EvenNum)
    
    def b := g(a) # we don't have to cast a to an EvenNum, it is already of that type
    
    def c := h(x)  # function h never raises an error
```
