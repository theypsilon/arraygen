use arraygen::Arraygen;

fn main() {
    read_prices();
    to_lowercase();
    call_trait_objects();
    implicit_select_all();
}

fn read_prices() {
    let prices = Prices {
        water: 1.0,
        oil: 3.0,
        tomato: 2.0,
        chocolate: 4.0,
    };

    println!(
        "Sum of all prices: {}",
        prices.get_all_prices().iter().sum::<f32>()
    );
}

#[derive(Arraygen)]
#[gen_array(fn get_all_prices: f32)]
struct Prices {
    #[in_array(get_all_prices)]
    pub water: f32,
    #[in_array(get_all_prices)]
    pub oil: f32,
    #[in_array(get_all_prices)]
    pub tomato: f32,
    #[in_array(get_all_prices)]
    pub chocolate: f32,
}

fn to_lowercase() {
    let mut person = Person {
        first_name: "ADa".into(),
        last_name: "LoVElaCE".into(),
    };

    for name in person.get_names().iter_mut() {
        **name = name.to_lowercase();
    }

    println!("Lowercase Ada name is: {}", person);
}

#[derive(Arraygen)]
#[gen_array(fn get_names: &mut String)]
struct Person {
    #[in_array(get_names)]
    pub first_name: String,
    #[in_array(get_names)]
    pub last_name: String,
}

impl std::fmt::Display for Person {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{} {}", self.first_name, self.last_name)
    }
}

fn call_trait_objects() {
    let animals = Animals {
        dogo: Dog {},
        tiger: Cat {},
        bacon: Pig {},
    };

    let talk = animals
        .get_animals()
        .iter()
        .map(|animal| animal.talk())
        .collect::<Vec<&'static str>>()
        .join(", ");

    println!("Animals say: {}", talk);
}

trait Animal {
    fn talk(&self) -> &'static str;
}
struct Dog {}
impl Animal for Dog {
    fn talk(&self) -> &'static str {
        "bark"
    }
}
struct Cat {}
impl Animal for Cat {
    fn talk(&self) -> &'static str {
        "meow"
    }
}
struct Pig {}
impl Animal for Pig {
    fn talk(&self) -> &'static str {
        "oink"
    }
}

#[derive(Arraygen)]
#[gen_array(fn get_animals: &dyn Animal)]
struct Animals {
    #[in_array(get_animals)]
    dogo: Dog,
    #[in_array(get_animals)]
    tiger: Cat,
    #[in_array(get_animals)]
    bacon: Pig,
}

fn implicit_select_all() {
    let prices = ImplicitPrices {
        water: 2.0,
        oil: 4.0,
        tomato: 3.0,
        chocolate: 5.0,
    };

    println!(
        "Sum of all implicit prices: {}",
        prices.get_all_prices().iter().sum::<f64>()
    );

    let animals = ImplicitAnimals {
        dogo: Dog {},
        tiger: Cat {},
        bacon: Pig {},
    };

    let talk = animals
        .get_animals()
        .iter()
        .map(|animal| animal.talk())
        .collect::<Vec<&'static str>>()
        .join(", ");

    println!("Implicit animals say: {}", talk);
}

#[derive(Arraygen)]
#[gen_array(fn get_all_prices: f64, implicit_select_all: f64)]
struct ImplicitPrices {
    pub water: f64,
    pub oil: f32,
    pub tomato: f32,
    pub chocolate: f32,
}

#[derive(Arraygen)]
#[gen_array(fn get_animals: &dyn Animal, implicit_select_all: Dog, Cat, Pig)]
struct ImplicitAnimals {
    bacon: Pig,
    dogo: Dog,
    tiger: Cat,
}
