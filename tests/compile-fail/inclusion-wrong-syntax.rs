#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub fn my_array: i32)]
struct Test{
    #[in_array(..er))] //~ERROR 'in_array' was used with the wrong syntax.
    foo: i32
}