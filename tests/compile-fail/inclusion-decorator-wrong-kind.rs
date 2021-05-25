#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub fn my_array: i32)]
struct Test{
    #[in_array(my_array { blabla })] //~ERROR 'in_array' doesn't allow 'blabla' as a decorator
    foo: i32
}