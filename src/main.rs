use image;
use num_complex::Complex64;
use ndarray::prelude::*;
use ndarray::par_azip;



fn main() -> Result<(), String> {
    let img_size = (1024, 1024);
    let out_path = "./out.png";
    let mut img = Array2::<u8>::zeros(img_size);

    par_azip!((index (y, x), pixel in &mut img) {
        let mut z = Complex64::new(x as f64/img_size.0 as f64, y as f64/img_size.1 as f64);
        z = z*4.-Complex64::new(2., 2.);

        *pixel = fractal_iterations(z, 255) as u8;
    });

    let data = img.as_slice().unwrap();
    let (width, height) = (img_size.0 as u32, img_size.1 as u32);

    image::save_buffer(out_path, data, width, height, image::ColorType::L8)
          .map_err(|e| e.to_string())
}

fn fractal_iterations(z0: Complex64, max_iter: usize) -> usize {
    let mut z = z0;

    for i in 0..max_iter {
        if z.norm_sqr() > 4. { return i }
        z = z*z+z0;
    }

    max_iter
}
