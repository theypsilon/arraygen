#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub fn my_array: i32)]
struct Test{
    #[in_array(my_array2 { cast })] //~ERROR 'in_array' can not decorate the given method because it wasn't declared with 'gen_array'
    foo: i32
}