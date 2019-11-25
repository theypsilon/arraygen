#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub fn my_array: i32)]
#[gen_array(pub fn another_array: i32)]
struct Test{
    #[in_array(my_array another_array)] //~ERROR 'in_array' shouldn't contain those tokens.
    foo: i32
}