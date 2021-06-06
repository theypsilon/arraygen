#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub fn my_array: i32)]
struct Test{
    #[in_array(my_array, my_array)] //~ERROR 10:26: 10:34: Field 'foo' is already included in gen_array method 'my_array'
    foo: i32
}