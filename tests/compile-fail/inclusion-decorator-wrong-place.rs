#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub fn my_array: i32)]
struct Test{
    #[in_array({ cast } my_array )] //~ERROR 10:16: 10:17: expected identifier
    foo: i32
}