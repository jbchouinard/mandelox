use structopt::StructOpt;

use mandelox::mandelbrot;
use mandelox::painter::Rainbow;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "1200")]
    width: i64,
    #[structopt(short, long, default_value = "1000")]
    height: i64,
    #[structopt(short, long, default_value = "out.png")]
    output: String,
}

fn main() {
    let opt = Opt::from_args();
    mandelbrot(opt.width, opt.height)
        .paint(Rainbow, 100)
        .save(opt.output)
        .expect("failed to save image");
}
