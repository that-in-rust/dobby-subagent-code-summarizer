//! Simple test file for summarization validation

fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

fn multiply_numbers(x: i32, y: i32) -> i32 {
    x * y
}

struct Calculator {
    value: i32,
}

impl Calculator {
    fn new(value: i32) -> Self {
        Calculator { value }
    }

    fn add(&mut self, other: i32) {
        self.value += other;
    }

    fn get_value(&self) -> i32 {
        self.value
    }
}

fn main() {
    let mut calc = Calculator::new(10);
    calc.add(5);

    let result = add_numbers(calc.get_value(), 3);
    println!("Final result: {}", result);
}
