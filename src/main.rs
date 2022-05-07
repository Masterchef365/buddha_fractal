// For reading and opening files
use anyhow::Result;
use rand::distributions::Uniform;
use rand::prelude::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Output path
    #[structopt(default_value = "out.png")]
    out_path: PathBuf,

    /// Image width
    #[structopt(short, long, default_value = "1920")]
    width: usize,

    /// Image height
    #[structopt(short, long, default_value = "1080")]
    height: usize,

    /// Image center x position
    #[structopt(short = "cx", long, default_value = "-0.5")]
    center_x: f32,

    /// Image center y position
    #[structopt(short = "cy", long, default_value = "0.0")]
    center_y: f32,

    /// Image scale
    #[structopt(short, long, default_value = "1.0")]
    scale: f32,

    /// Total iterations
    #[structopt(short, long, default_value = "10000000")]
    iters: usize,

    /// Max steps per iteration 
    #[structopt(short, long, default_value = "1500")]
    steps: usize,
}

fn mandelbrot(x: f32, y: f32) -> impl Iterator<Item = (f32, f32)> {
    let (mut a, mut b) = (0., 0.);
    std::iter::from_fn(move || {
        let tmp = a * a - b * b + x;
        b = 2. * a * b + y;
        a = tmp;

        if a * a + b * b > 2. * 2. {
            None
        } else {
            Some((a, b))
        }
    })
}

fn main() -> Result<()> {
    let args = Opt::from_args();
    let mut image_data = vec![0_u8; args.width * args.height];

    //let scale = |x: f32| (x * 2. - 1.) * args.scale;
    let scale = |x: f32| ((x / args.scale) + 1.) / 2.;

    let aspect = args.width as f32 / args.height as f32;

    let mut steps = Vec::with_capacity(args.steps);
    for (idx, (x, y)) in disc(2.).take(args.iters).enumerate() {
        steps.clear();
        steps.extend(mandelbrot(x, y).take(args.steps));

        if idx % 100_000 == 0 {
            println!("{}/{} ({}%)", idx, args.iters, idx * 100 / args.iters);
        }

        if steps.len() != args.steps {
            for (x, y) in steps.drain(..) {
                let x = scale((x - args.center_x) / aspect) * args.width as f32;
                let y = scale(y - args.center_y) * args.height as f32;

                let bound_x = x >= 0. && x < args.width as f32;
                let bound_y = y >= 0. && y < args.height as f32;

                if bound_x && bound_y {
                    let idx = x as usize + y as usize * args.width;
                    image_data[idx] = image_data[idx].saturating_add(1);
                }
            }
        }
    }

    write_png(
        &args.out_path,
        &image_data,
        args.width as u32,
        args.height as u32,
    )
}

fn disc(radius: f32) -> impl Iterator<Item = (f32, f32)> {
    let unif = Uniform::new(-radius, radius);
    unif.sample_iter(rand::thread_rng())
        .zip(unif.sample_iter(rand::thread_rng()))
        .filter(move |(x, y)| x * x + y * y < radius * radius)
}

fn write_png(path: &Path, data: &[u8], width: u32, height: u32) -> Result<()> {
    let file = File::create(path)?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&data)?; // Save
                                     //
    Ok(())
}
