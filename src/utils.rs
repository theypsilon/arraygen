pub trait OptionExtensions<T> {
    fn insert_stable(&mut self, value: T) -> &mut T;
}

impl<T> OptionExtensions<T> for Option<T> {
    fn insert_stable(&mut self, value: T) -> &mut T {
        *self = Some(value);
        match self {
            Some(value) => value,
            None => unreachable!(),
        }
    }
}
