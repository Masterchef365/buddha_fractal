// For reading and opening files
use anyhow::Result;
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
}

fn mandelbrot(x: f32, y: f32, iters: usize) -> Option<usize> {
    let (mut a, mut b) = (x, y);
    for i in 1..=iters {
        let tmp = a * a - b * b + x;
        b = 2. * a * b + y;
        a = tmp;

        if a * a + b * b > 2. * 2. {
            return Some(i);
        }
    }

    None
}

fn main() -> Result<()> {
    let args = Opt::from_args();
    let mut image_data = vec![0_u8; args.width * args.height];

    let scale = |x: f32| (x * 2. - 1.) * args.scale;

    let aspect = args.width as f32 / args.height as f32;

    for (row_idx, row) in image_data.chunks_exact_mut(args.width).enumerate() {
        let y = scale(row_idx as f32 / args.height as f32) + args.center_y;

        for (col_idx, elem) in row.iter_mut().enumerate() {
            let x = scale((col_idx as f32 / args.width as f32)) * aspect + args.center_x;

            let m = mandelbrot(x, y, 255).unwrap_or(0);

            *elem = m as u8;
        }
    }

    write_png(
        &args.out_path,
        &image_data,
        args.width as u32,
        args.height as u32,
    )
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
