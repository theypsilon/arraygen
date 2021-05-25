#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub fn my_array: i32)]
struct Test{
    #[in_array(my_array, my_array)] //~ERROR Field 'foo' is already included in method 'my_array', no need to repeat it
    foo: i32
}