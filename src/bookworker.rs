use super::orderbook;

pub fn print_level() {
    let level = orderbook::Level {
        exchange: String::from("Binance"),
        price: 100.0,
        amount: 5.0,
    };
    println!("Level: {:?}", level);
}
