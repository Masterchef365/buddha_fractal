// For reading and opening files
use anyhow::Result;
use rand::distributions::Uniform;
use rand::prelude::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::thread::{available_parallelism, JoinHandle};
use structopt::StructOpt;

type Image = Vec<u16>;

#[derive(Debug, Clone, StructOpt)]
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

    /// Cutoff disc radius. Optimally 2.0, 
    /// but default to 3.0 for presentation purposes
    #[structopt(long, default_value = "3.0")]
    disc: f32,

    /// Divide all bin counts by this number for image output
    #[structopt(short, long, default_value = "2")]
    div: u16,

    /// Total iterations
    #[structopt(short, long, default_value = "10000000")]
    iters: usize,

    /// Max steps per iteration
    #[structopt(short, long, default_value = "1500")]
    steps: usize,
}

fn mandelbrot(x: f32, y: f32, disc: f32) -> impl Iterator<Item = (f32, f32)> {
    let (mut a, mut b) = (0., 0.);
    let r = disc * disc;
    std::iter::from_fn(move || {
        let tmp = a * a - b * b + x;
        b = 2. * a * b + y;
        a = tmp;

        if a * a + b * b > r {
            None
        } else {
            Some((a, b))
        }
    })
}

fn worker_thread(args: Opt, iters: usize) -> Image {
    let mut image_data = vec![0_u16; args.width * args.height];

    // Image framing
    let scale = |x: f32| ((x / args.scale) + 1.) / 2.;
    let aspect = args.width as f32 / args.height as f32;

    // Save all steps taken
    let mut steps = Vec::with_capacity(args.steps);

    for (idx, (x, y)) in disc(args.disc).take(iters).enumerate() {
        steps.clear();
        steps.extend(mandelbrot(x, y, args.disc).take(args.steps));

        // Print progress
        if idx % 100_000 == 0 {
            println!("{}/{} ({}%)", idx, iters, idx * 100 / iters);
        }

        // If the function diverged...
        if steps.len() != args.steps {
            for (x, y) in steps.drain(..) {
                // Find position in image
                let x = scale((x - args.center_x) / aspect) * args.width as f32;
                let y = scale(y - args.center_y) * args.height as f32;

                // Bounds check
                let bound_x = x >= 0. && x < args.width as f32;
                let bound_y = y >= 0. && y < args.height as f32;

                // Write to image
                if bound_x && bound_y {
                    let idx = x as usize + y as usize * args.width;
                    image_data[idx] = image_data[idx].saturating_add(1);
                }
            }
        }
    }

    image_data
}

fn main() -> Result<()> {
    let args = Opt::from_args();

    // Divide work
    let n_workers = available_parallelism().map(|v| v.get()).unwrap_or(1);
    let iters_per_worker = args.iters / n_workers;

    // Spawn workers
    let workers: Vec<JoinHandle<Image>> = (0..n_workers)
        .map(|_| {
            let args = args.clone();
            std::thread::spawn(move || worker_thread(args, iters_per_worker))
        })
        .collect();

    // Collect results
    let mut images: Vec<Image> = workers
        .into_iter()
        .map(|w| w.join().expect("Worker failed"))
        .collect();

    // Sum images
    let mut out_image = images.pop().unwrap();
    for image in images {
        out_image
            .iter_mut()
            .zip(image.iter())
            .for_each(|(o, i)| *o = o.saturating_add(*i));
    }

    // Determine coloring
    let out_image: Vec<u8> = out_image
        .into_iter()
        .map(|i| (i / args.div).min(u8::MAX as u16) as u8)
        .collect();

    // Write results
    write_png(
        &args.out_path,
        &out_image,
        args.width as u32,
        args.height as u32,
    )
}

/// Produce random points on the unit disc with the given radius
fn disc(radius: f32) -> impl Iterator<Item = (f32, f32)> {
    let unif = Uniform::new(-radius, radius);
    unif.sample_iter(rand::thread_rng())
        .zip(unif.sample_iter(rand::thread_rng()))
        .filter(move |(x, y)| x * x + y * y < radius * radius)
}

/// Write a grayscale PNG at the given path
fn write_png(path: &Path, data: &[u8], width: u32, height: u32) -> Result<()> {
    let file = File::create(path)?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&data)?;

    Ok(())
}
