extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)] //~WARNING 'in_array' shouldn't contain these tokens.
struct Empty{}

fn main() {
    let _ = Empty{};    
}

// @TODO Not active, activate it when is possible on stable.