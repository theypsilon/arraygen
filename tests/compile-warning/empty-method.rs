extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(fn empty: i32)] //~WARNING Method 'empty' returns an empty array.
struct Test{}

fn main() {
    let test = Test{};
    assert_eq!(test.empty().len(), 0);
}

// @TODO Not active, activate it when is possible on stable.