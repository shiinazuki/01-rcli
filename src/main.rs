fn main() {
    println!("Hello, rust!!!");
}

#[cfg(test)]
mod tests {

    #[test]
    fn add() {
        println!("add");
    }
}
