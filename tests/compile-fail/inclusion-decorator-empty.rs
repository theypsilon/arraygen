#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub fn my_array: i32)]
struct Test{
    #[in_array(my_array2 { })] //~ERROR 10:26: 10:29: unexpected end of input, expected identifier
    foo: i32
}