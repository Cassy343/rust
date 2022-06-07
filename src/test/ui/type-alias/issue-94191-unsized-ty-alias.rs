// Check for improved type alias diagnostics

struct Container<T>(T);

type UnsizedContainer = Container<str>;
type ContainerAlias<T> = Container<T>;

fn takes_container(_: UnsizedContainer) {}
//~^ ERROR the size for values of type `str` cannot be known at compilation time [E0277]

fn takes_generic_container<T>(_: ContainerAlias<T>)
//~^ ERROR the size for values of type `T` cannot be known at compilation time [E0277]
where
    T: ?Sized,
{
}

fn main() {}
