
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufReader, BufRead};
use std::io;
use std::io::Write;
use std::env;
use rustfft;
use rustfft::num_complex::Complex32;
//use bitvec::vec::BitVec;

struct Config {
    slice_size: usize,
    num_bands: usize,
}

fn main() {
    let config = Config{
        slice_size: 16537,
        num_bands: 33,
        
    };
    let mut args = env::args();
    let _ = args.next();
    let first_file = args.next().expect("Argument missing; must provide first input file");
    let second_file= args.next().expect("Argument missing; must provide second input");
    let first_out = args.next().expect("Argument missing; must provide first output");
    let second_out = args.next().expect("Argument missing; must provide second output");

   let mut first_data = read_file(&first_file);
   let mut second_data = read_file(&second_file);
   let offset = align(&first_data, &second_data);
   second_data.drain(0..offset);
   first_data.truncate(first_data.len()-offset);
   let first_bits = fingerprint(first_data, &config);
   let second_bits=fingerprint(second_data, &config);
   write_txt(&first_out, &first_bits);
   write_txt(&second_out, &second_bits);
   println!("Vecs have truncated length {} and are different in {} locations", first_bits.len(), distance(&first_bits, &second_bits));
   /*for bit in &bits {
      print!("{}", bit);
   } 
   io::stdout().flush().unwrap()*/
   //println!("{:#?}", hanning_window(&test, test.len()));
  // println!("{:#?}", fourier(hanning_window(&test, test.len()), test.len()));

}
fn distance(first: &Vec<u8>, second: &Vec<u8>)->usize{
    let mut sum =0;
    for i in 0..first.len(){
        if first[i]!=second[i] {sum = sum+1;}
    }
    sum
}
fn align(first: &Vec<i16>, second: &Vec<i16>)->usize {
    let mut offset = (0, 0);
    let max_len = if first.len() < second.len() { first.len() } else {second.len()};
    for i in 0..max_len{
        let mut cross_corr = 0;
        for j in 0..(max_len-i){
            cross_corr=cross_corr+(first[j]*second[i+j]);
        }
        if cross_corr> offset.1 {
            offset = (i,cross_corr)
        }
    }
    offset.0
}
fn hanning_window(input: &Vec<i16>, window_len: usize)->Vec<f32>{
    let mut i = 0;
    let mut to_ret = Vec::with_capacity(input.len());
    while i<input.len() {
        //I think some of these unit conversions are unnecessary, but unclear which.
        let hann=0.5f32 * (1f32-(2f32*std::f32::consts::PI*(i%window_len) as f32/window_len as f32).cos());
        to_ret.push(hann * input[i] as f32);
        i=i+1;
    }
   // println!("Windowed: {:#?}", to_ret);
    to_ret
}
fn read_file(filename: &str)->Vec<i16>{
    let f = File::open(filename).expect("Failed to open file.");
    let reader = BufReader::new(f);
    let mut to_ret=Vec::new();
    for line in reader.lines() {
        //to_ret.push(i16::from_str_radix(&line.unwrap(), 10).unwrap());
        let val = &line.unwrap();
        match i16::from_str_radix(val, 10) {
            Ok(t) => to_ret.push(t),
            Err(e) => println!("In read file,{} {}", val, e)
        };
    } 
    to_ret
}
fn fourier(data:Vec<f32>, slice_size: usize)->Vec<Complex32>{
   // println!("Starting Fourier");
    
    
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
//Let's define a struct that has al the config data; 5 usizes in a row is super clunky.
fn fingerprint(mut data: Vec<i16>, config: &Config)->Vec<u8>{
    //Probably should end up with this back inside the call to fourier, and just replace calls to data.len() with transformed.len()
    if {data.len()%config.slice_size !=0} {
        data.truncate(data.len()-data.len()%config.slice_size);
    }
    let frames = data.len()/config.slice_size;
   // println!("Frames: {}", frames);
    //let band_width = (config.max_freq-config.min_freq)/config.num_bands;
    let mut energy_matrix = Vec::with_capacity(frames);
    let mut i = 0;
    let transformed = fourier(hanning_window(&data, config.slice_size), config.slice_size);
    while i<data.len(){
        let mut row = vec![0f32; config.num_bands];
        let frame_end = i+ config.slice_size;
        while i<frame_end&&i<transformed.len(){
            let curr_band =((i% config.slice_size) as f32/config.slice_size as f32 *config.num_bands as f32) as usize;
           // println!("On iter {}, band is {}", i, curr_band);
            let to_add = transformed[i].norm_sqr();
            row[curr_band] = row[curr_band]+to_add;
            i = i+1;
        }
        energy_matrix.push(row);
        //println!("Row built, value of i is {} and next frame ends at {}", i, frame_end);
    }
    //println!("Energy matrix built");
    //In principle, we here have a full energy matrix.
    //In practice, I'm short several hundred bits.
    //println!("There are {} rows of len {}", energy_matrix.len(), energy_matrix[0].len());
    
    i = 1;
    let mut to_ret = Vec::new();
    while i<energy_matrix.len(){
        let mut j = 0;
        while j< config.num_bands-1 {
            to_ret.push(if energy_matrix[i][j] - energy_matrix[i][j+1] - (energy_matrix[i-1][j]-energy_matrix[i-1][j+1]) > 0.0 {
                1u8
            }else{
               0u8
            });
            j = j+1;
        };
        i=i+1;  
    };
 //  println!("Fingerprint built");
    to_ret
    
}

fn write_txt(filename: &str, buf: &Vec<u8>)->(){
    //let mut target = File::create(filename).unwrap();
    let mut target = OpenOptions::new().append(true).create(true).open(filename).unwrap();
     for i in 0..buf.len() {
         if{i%2==0}{
         target.write_all(buf[i].to_string().as_bytes()).unwrap();
         //Line breaks appear to be undesirable
         target.write_all("\n".as_bytes()).unwrap();
         }
     }
 }
 