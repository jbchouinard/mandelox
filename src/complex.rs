use num::complex::Complex;

pub type C<T> = Complex<T>;

pub fn c(re: f64, im: f64) -> C<f64> {
    Complex::new(re, im)
}

pub fn cr(re: f64) -> C<f64> {
    c(re, 0.0)
}

pub fn ci(im: f64) -> C<f64> {
    c(0.0, im)
}
