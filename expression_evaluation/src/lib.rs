use std::error::Error;

#[derive(PartialEq, Debug)]
enum ExprType {
    Logical,
    Numerical,
}

pub struct Config {
    expr_type: ExprType,
    expr: String,
}

/// builds the arguments from cli arguments
impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next(); // skip the first argument which is the program name

        let expr_type = if let Some(arg) = args.next() {
            if arg == "logical" {
                ExprType::Logical
            } else if arg == "numerical" {
                ExprType::Numerical
            } else {
                return Err("Not a supported type");
            }
        } else {
            return Err("Didn't get a type");
        };

        let expr = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get an expression"),
        };

        Ok(Config { expr_type, expr })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    match config.expr_type {
        ExprType::Logical => {
            let mut logic_expr = logical_expression::Expression::new(&config.expr);

            match logic_expr.eval() {
                Ok(result) => {
                    println!("Logical result = {:?}", result)
                }
                Err(..) => {
                    println!("Error in your expression")
                }
            };
        }
        ExprType::Numerical => {
            let mut num_expr = numerical_expression::Expression::new(&config.expr);

            match num_expr.eval() {
                Ok(result) => {
                    println!("Calculation result = {:?}", result)
                }
                Err(..) => {
                    println!("Error in your expression")
                }
            };
        }
    };

    Ok(())
}
