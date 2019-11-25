#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub fn my_array: i32)]
struct Test{
    #[in_array(another_array)] //~ERROR Method 'another_array' was not declared with the attribute 'gen_array' at struct level.
    foo: i32
}