#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub fn my_array: i32)]
struct Test{
    #[in_array(another_array)] //~ERROR 10:16: 10:29: gen_array method 'another_array' not present but used by field 'foo'
    foo: i32
}