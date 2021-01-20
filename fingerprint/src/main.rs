
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::env;
use rustfft;
use rustfft::num_complex::Complex32;
fn main() {
    let mut args = env::args();
    let _ = args.next();
    let filename = args.next().expect("Argument missing; must provide filename");

   let test = read_file(&filename);
   println!("{:#?}", hanning_window(&test, test.len()));
   println!("{:#?}", fourier(hanning_window(&test, test.len()), test.len()));
}


fn hanning_window(input: &Vec<i16>, window_len: usize)->Vec<f32>{
    let mut i = 0;
    let mut to_ret = Vec::with_capacity(input.len());
    while i<input.len() {
        //I think some of these unit conversions are unnecessary, but unclear which.
        let hann=0.5f32 * (1f32-(2f32*std::f32::consts::PI*(i%window_len) as f32/window_len as f32).cos());
        to_ret.push(hann * *input.get(i).unwrap() as f32);
        i=i+1;
    }
    to_ret
}
fn read_file(filename: &str)->Vec<i16>{
    let f = File::open(filename).expect("Failed to open file.");
    let reader = BufReader::new(f);
    let mut to_ret=Vec::new();
    for line in reader.lines() {
        //to_ret.push(i16::from_str_radix(&line.unwrap(), 10).unwrap());
        match i16::from_str_radix(&line.unwrap(), 10) {
            Ok(t) => to_ret.push(t),
            Err(e) => println!("{}", e)
        };
    } 
    to_ret
}
fn fourier(data:Vec<f32>, slice_size: usize)->Vec<Complex32>{
    let mut planner = rustfft::FftPlanner::new();
    let plan = planner.plan_fft_forward(slice_size);
  //  let mut buffer: Vec<Complex<f32>> = data.iter().map(|x| Complex::new(&x, 0f32)).collect();
  //For some reason, can't collect after the map; at some point will figure out why, but for now will ignore
    let mut buffer = Vec::with_capacity(data.len());
    let mut i = 0;
    while i<data.len() {
        buffer.push(Complex32::new(*data.get(i).unwrap(), 0.0));
        i=i+1;
    }
    plan.process(&mut buffer);
    buffer.to_vec()
}
fn fingerprint(data: &Vec<i16>, slice_size: usize, maxfreq:usize, minfreq: usize, bandwidth: usize)->Vec<bool>{
    todo!()
}