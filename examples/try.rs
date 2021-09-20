fn main() {
    println!("{}", test());
}

fn test() -> &'static str {
    let hoge = String::from("aaaaaaa");
    hoge.as_str()
}
