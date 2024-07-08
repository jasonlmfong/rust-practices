use numerical_expression::Expression;

fn main() {
    let src = "((325 * 12) + 106) / 4 + 23";
    let mut expr = Expression::new(src);
    match expr.eval() {
        Ok(result) => {
            println!("result = {:?}", result)
        }
        Err(..) => {
            println!("error in your expression")
        }
    };
}
