def do_something(x: int):
    print(f"hello world {x}")

my_resource = 10

with my_resource as other:
    do_something(other)

with my_resource as yet_another:
    do_something(yet_another)

with my_resource:
    do_something(my_resource)
