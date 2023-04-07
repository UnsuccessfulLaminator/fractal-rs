use image;
use num_complex::Complex64;
use ndarray::prelude::*;
use ndarray::par_azip;
use std::process::exit;



fn main() {
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

    let mut img = Array3::<u8>::zeros(img_size);
    let mut iters = Array2::<usize>::zeros((img_size.0, img_size.1));
    let mut bins = Array1::<f64>::zeros(max_iterations+1);
    
    iterate_points(img_center, img_minor_radius, max_iterations, &mut iters.view_mut());
    gen_histogram(iters.view(), &mut bins.view_mut());

    par_azip!((mut pixel in img.rows_mut(), &iter in &iters) {
        lerp_colors(&colors, bins[iter], &mut pixel);
    });
    
    match save_image(&out_path, img.view()) {
        Ok(_) => println!("Saved to {out_path}"),
        Err(e) => {
            eprintln!("Could not save to {out_path}. Error: {}", e.to_string());
            exit(1);
        }
    };
}

fn save_image<D: Dimension>(path: &str, image: ArrayView<u8, D>) -> image::ImageResult<()> {
    let color_type = match image.ndim() {
        2 => image::ColorType::L8,
        3 => match image.len_of(Axis(2)) {
            1 => image::ColorType::L8,
            2 => image::ColorType::La8,
            3 => image::ColorType::Rgb8,
            4 => image::ColorType::Rgba8,
            _ => panic!("Image array pixels must have 1 to 4 components!"),
        },
        _ => panic!("Image array must have 2 or 3 dimensions!"),
    };

    let (width, height) = (image.len_of(Axis(0)) as u32, image.len_of(Axis(1)) as u32);
    let mut transposed = image.view();

    transposed.swap_axes(0, 1);

    let contiguous = transposed.as_standard_layout();
    let data = contiguous.as_slice().unwrap();

    image::save_buffer(path, data, width, height, color_type)
}

fn gen_histogram<D: Dimension>(values: ArrayView<usize, D>, out: &mut ArrayViewMut1<f64>) {
    out.fill(0.);

    for &val in &values { out[val] += 1.; } // Frequencies of different values
    for i in 0..out.len()-1 { out[i+1] += out[i]; } // Make cumulative
    
    *out /= out[out.len()-1]; // As a fraction of the total
}

fn iterate_points(
    center: Complex64, minor_radius: f64, max_iterations: usize,
    out: &mut ArrayViewMut2<usize>
) {
    let (width, height) = out.dim();
    let aspect = width as f64/height as f64;
    let re_radius = if width < height { minor_radius } else { minor_radius*aspect };
    let im_radius = re_radius/aspect;
    let re_range = (center.re-re_radius, center.re+re_radius);
    let im_range = (center.im-im_radius, center.im+im_radius);
    
    par_azip!((index (x, y), iter in out) {
        let z = Complex64::new(
            change_range(x as f64, 0., width as f64, re_range.0, re_range.1),
            change_range(y as f64, 0., height as f64, im_range.0, im_range.1)
        );

        *iter = fractal_iterations(z, max_iterations);
    });
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
