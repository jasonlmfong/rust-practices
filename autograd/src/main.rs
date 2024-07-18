use std::cell::RefCell;

#[derive(Clone, Copy)]
struct Node {
    // since we assume operations are binary (take in 2 vars)
    weights: [f64; 2],
    deps: [usize; 2], // dependency indices
}

pub struct Tape {
    nodes: RefCell<Vec<Node>>,
}

impl Tape {
    pub fn new() -> Self {
        Tape {
            nodes: RefCell::new(Vec::new()),
        }
    }

    pub fn var<'t>(&'t self, value: f64) -> Var<'t> {
        Var {
            tape: self,
            value: value,
            index: self.push_scalar(),
        }
    }

    fn len(&self) -> usize {
        self.nodes.borrow().len()
    }

    fn push_scalar(&self) -> usize {
        let mut nodes = self.nodes.borrow_mut();
        let len = nodes.len();
        nodes.push(Node {
            weights: [0.0, 0.0],
            deps: [len, len],
        });
        len
    }

    fn push_unary(&self, dep0: usize, weight0: f64) -> usize {
        let mut nodes = self.nodes.borrow_mut();
        let len = nodes.len();
        nodes.push(Node {
            weights: [weight0, 0.0],
            deps: [dep0, len],
        });
        len
    }

    fn push_binary(&self, dep0: usize, weight0: f64, dep1: usize, weight1: f64) -> usize {
        let mut nodes = self.nodes.borrow_mut();
        let len = nodes.len();
        nodes.push(Node {
            weights: [weight0, weight1],
            deps: [dep0, dep1],
        });
        len
    }
}

#[derive(Clone, Copy)]
pub struct Var<'t> {
    tape: &'t Tape, //Wengert list
    index: usize,
    value: f64,
}

impl<'t> Var<'t> {
    pub fn grad(&self) -> Grad {
        let len = self.tape.len();
        let nodes = self.tape.nodes.borrow();

        // allocate the array of derivatives (specifically: adjoints)
        let mut derivs = vec![0.0; len];

        // seed
        derivs[self.index] = 1.0;

        // traverse the tape in reverse
        for i in (0..len).rev() {
            let node = nodes[i];
            let deriv = derivs[i];

            // update the adjoints for its parent nodes
            for j in 0..2 {
                derivs[node.deps[j]] += node.weights[j] * deriv;
            }
        }

        Grad { derivs: derivs }
    }

    ///// line break

    fn invert(self) -> Self {
        Var {
            tape: self.tape,
            value: 1.0 / self.value,
            index: self
                .tape
                .push_unary(self.index, (-1.0) / (self.value * self.value)),
        }
    }

    ///// line break

    pub fn sqrt(self) -> Self {
        Var {
            tape: self.tape,
            value: self.value.sqrt(),
            index: self
                .tape
                .push_unary(self.index, 1.0 / (2.0 * self.value.sqrt())),
        }
    }

    pub fn sin(self) -> Self {
        Var {
            tape: self.tape,
            value: self.value.sin(),
            index: self.tape.push_unary(self.index, self.value.cos()),
        }
    }

    pub fn cos(self) -> Self {
        Var {
            tape: self.tape,
            value: self.value.cos(),
            index: self.tape.push_unary(self.index, -self.value.sin()),
        }
    }

    pub fn exp(self) -> Self {
        Var {
            tape: self.tape,
            value: self.value.exp(),
            index: self.tape.push_unary(self.index, self.value.exp()),
        }
    }

    pub fn log(self) -> Self {
        Var {
            tape: self.tape,
            value: self.value.ln(),
            index: self.tape.push_unary(self.index, 1.0 / self.value),
        }
    }
}

impl<'t> ::std::ops::Add for Var<'t> {
    type Output = Var<'t>;
    fn add(self, other: Var<'t>) -> Self::Output {
        assert_eq!(self.tape as *const Tape, other.tape as *const Tape);
        Var {
            tape: self.tape,
            value: self.value + other.value,
            index: self.tape.push_binary(self.index, 1.0, other.index, 1.0),
        }
    }
}

impl<'t> ::std::ops::Sub for Var<'t> {
    type Output = Var<'t>;
    fn sub(self, other: Var<'t>) -> Self::Output {
        assert_eq!(self.tape as *const Tape, other.tape as *const Tape);
        Var {
            tape: self.tape,
            value: self.value - other.value,
            index: self.tape.push_binary(self.index, 1.0, other.index, 1.0),
        }
    }
}

impl<'t> ::std::ops::Mul for Var<'t> {
    type Output = Var<'t>;
    fn mul(self, other: Var<'t>) -> Self::Output {
        assert_eq!(self.tape as *const Tape, other.tape as *const Tape);
        Var {
            tape: self.tape,
            value: self.value * other.value,
            index: self
                .tape
                .push_binary(self.index, other.value, other.index, self.value),
        }
    }
}

impl<'t> ::std::ops::Div for Var<'t> {
    type Output = Var<'t>;
    fn div(self, other: Var<'t>) -> Self::Output {
        assert_ne!(other.value, 0.0);
        assert_eq!(self.tape as *const Tape, other.tape as *const Tape);
        self * other.invert()
    }
}

pub struct Grad {
    derivs: Vec<f64>,
}

impl Grad {
    pub fn wrt<'t>(&self, var: Var<'t>) -> f64 {
        self.derivs[var.index]
    }
}

fn main() {
    let t = Tape::new();

    // modify the values here
    // let x = t.var(1.0);
    // let y = t.var(3.0);
    // let z = x.exp() + y.log();

    let grad = z.grad();

    println!("z = {}", z.value);
    println!("∂z/∂x = {}", grad.wrt(x));
    println!("∂z/∂y = {}", grad.wrt(y));
}

#[cfg(test)]
mod tests {
    use super::Tape;

    #[test]
    fn x_times_y_plus_sin_x() {
        let t = Tape::new();
        let x = t.var(0.5);
        let y = t.var(4.2);
        let z = x * y + x.sin();
        let grad = z.grad();
        assert!((z.value - 2.579425538604203).abs() <= 1e-15);
        assert!((grad.wrt(x) - (y.value + x.value.cos())).abs() <= 1e-15);
        assert!((grad.wrt(y) - x.value).abs() <= 1e-15);
    }

    #[test]
    fn x_minus_x_div_by_y() {
        let t = Tape::new();
        let x = t.var(1.0);
        let y = t.var(4.0);
        let z = x - x / y;
        let grad = z.grad();
        assert!((z.value - 0.75).abs() <= 1e-15);
        assert!((grad.wrt(x) - 1.25).abs() <= 1e-15);
        assert!((grad.wrt(y) - (-0.0625)).abs() <= 1e-15);
    }

    #[test]
    fn exp_x_plus_ln_y() {
        let t = Tape::new();
        let x = t.var(1.0);
        let y = t.var(3.0);
        let z = x.exp() + y.log();
        let grad = z.grad();
        assert!((z.value - 3.8168941171271547).abs() <= 1e-15);
        assert!((grad.wrt(x) - 2.718281828459045).abs() <= 1e-15);
        assert!((grad.wrt(y) - 0.333333333333333).abs() <= 1e-15);
    }
}
