def do_something(x: int):
    print(f"hello world {x}")

my_resource: int = 10

with my_resource as other:
    __mamba_other_existed = "other" in locals()
    __mamba_other_saved = other if __mamba_other_existed else None
    do_something(other)
    if __mamba_other_existed:
        other = __mamba_other_saved
    else:
        del other

with my_resource as yet_another:
    __mamba_yet_another_existed = "yet_another" in locals()
    __mamba_yet_another_saved = yet_another if __mamba_yet_another_existed else None
    do_something(yet_another)
    if __mamba_yet_another_existed:
        yet_another = __mamba_yet_another_saved
    else:
        del yet_another

with my_resource:
    do_something(my_resource)
