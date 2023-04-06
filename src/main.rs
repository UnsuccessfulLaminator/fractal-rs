use image;
use num_complex::Complex64;
use ndarray::prelude::*;
use ndarray::par_azip;



fn main() -> Result<(), String> {
    // Set up these parameters for image generation
    let out_path = "./out.png";
    let img_size = (1366, 768, 3);
    let img_center = Complex64::new(-0.743643135, 0.131825963);
    let img_minor_radius = 0.00001;
    let max_iterations = 500;
    let colors = [
        array![1, 4, 13],
        array![1, 16, 39],
        array![8, 57, 100]
    ];

    // These are calculated 
    let re_radius = if img_size.0 < img_size.1 { img_minor_radius as f64 }
                    else { img_minor_radius as f64*img_size.0 as f64/img_size.1 as f64 };
    let im_radius = re_radius*img_size.1 as f64/img_size.0 as f64;
    let re_range = (img_center.re-re_radius, img_center.re+re_radius);
    let im_range = (img_center.im-im_radius, img_center.im+im_radius);
    
    let mut iters = Array2::<usize>::zeros((img_size.0, img_size.1));
    let mut bins = Array1::<f64>::zeros(max_iterations+1);
    let mut img = Array3::<u8>::zeros(img_size);
    
    azip!((index (x, y), iter in &mut iters) {
        let z = Complex64::new(
            change_range(x as f64, 0., img_size.0 as f64, re_range.0, re_range.1),
            change_range(y as f64, 0., img_size.1 as f64, im_range.0, im_range.1)
        );

        *iter = fractal_iterations(z, max_iterations);
        bins[*iter] += 1.;
    });
    
    for i in 0..max_iterations { bins[i+1] += bins[i]; }
    bins /= bins[max_iterations];

    par_azip!((mut pixel in img.rows_mut(), &iter in &iters) {
        lerp_colors(&colors, bins[iter], &mut pixel);
    });
    
    img.swap_axes(0, 1);
    
    let img = img.as_standard_layout();
    let data = img.as_slice().unwrap();
    let (width, height) = (img_size.0 as u32, img_size.1 as u32);

    image::save_buffer(out_path, data, width, height, image::ColorType::Rgb8)
          .map_err(|e| e.to_string())
}

fn lerp_colors(colors: &[Array1<u8>], value: f64, out: &mut ArrayViewMut1<u8>) {
    let scaled = value*(colors.len()-1) as f64;
    let idx = (scaled.floor() as usize).clamp(0, colors.len()-2);
    let frac = scaled-idx as f64;
    let (start_color, end_color) = (&colors[idx], &colors[idx+1]);

    azip!((o in out, &a in start_color, &b in end_color) {
        *o = lerp(a as f64, b as f64, frac) as u8;
    });
}

fn lerp(x1: f64, x2: f64, fraction: f64) -> f64 {
    x1+(x2-x1)*fraction
}

fn change_range(x: f64, start1: f64, end1: f64, start2: f64, end2: f64) -> f64 {
    start2+(x-start1)*(end2-start2)/(end1-start1)
}

fn fractal_iterations(z0: Complex64, max_iter: usize) -> usize {
    let mut z = z0;

    for i in 0..max_iter {
        if z.norm_sqr() > 4. { return i }
        z = z*z+z0;
    }

    max_iter
}
