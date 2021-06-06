#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub fn my_array: i32)]
struct Test{
    #[in_array(my_array { blabla })] //~ERROR 10:27: 10:33: in_array doesn't allow 'blabla' as decorator
    foo: i32
}