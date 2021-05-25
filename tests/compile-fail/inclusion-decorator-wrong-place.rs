#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub fn my_array: i32)]
struct Test{
    #[in_array({ cast } my_array )] //~ERROR 'in_array' decorators have to be placed after a method name
    foo: i32
}