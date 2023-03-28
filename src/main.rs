use image;
use num_complex::Complex64;
use ndarray::prelude::*;
use ndarray::{Zip, par_azip};



fn main() -> Result<(), String> {
    let img_size = (1024, 1024, 3);
    let out_path = "./out.png";
    let colors: Array2::<u8> = array![
        [237, 252, 27],
        [193, 50, 98],
        [11, 0, 116]
    ];
    let color_stops = Array1::<f64>::linspace(0., 1., colors.nrows());
    let max_iterations = 500;

    let mut img = Array3::<u8>::zeros(img_size);

    par_azip!((index (y, x), mut pixel in img.rows_mut()) {
        let mut z = Complex64::new(x as f64/img_size.0 as f64, y as f64/img_size.1 as f64);
        z = z*4.-Complex64::new(2., 2.);
        let iters = fractal_iterations(z, max_iterations);
        let iter_frac = iters as f64/(1.+max_iterations as f64);
        let color_idx = color_stops.iter().position(|&t| iter_frac < t).unwrap();
        let (a, b) = (color_stops[color_idx-1], color_stops[color_idx]);
        let interp = (iter_frac-a)/(b-a);

        Zip::from(&mut pixel)
            .and(colors.slice(s![color_idx-1, ..]))
            .and(colors.slice(s![color_idx, ..]))
            .for_each(|p, &c1, &c2| {
                *p = ((c1 as f64)*(1.-interp)+(c2 as f64)*interp) as u8;
            });
    });

    let data = img.as_slice().unwrap();
    let (width, height) = (img_size.0 as u32, img_size.1 as u32);

    image::save_buffer(out_path, data, width, height, image::ColorType::Rgb8)
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
