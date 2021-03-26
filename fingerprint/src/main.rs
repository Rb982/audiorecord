#![allow(dead_code)]
#![allow(unused_imports)]
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufRead, Read, Write, Result};
use std::io;
use std::env;
use rustfft;
use rustfft::num_complex::Complex32;
use std::net::{TcpStream, TcpListener};
mod rec;
mod reed_solomon;
use std::convert::TryInto;
use sha2;
use sha2::digest::Digest;
use std::mem::size_of;
//use bitvec::vec::BitVec;

struct Config {
    slice_size: usize,
    num_bands: usize,
    rec_frames: usize,
    pair_frames: usize,
    key_len: usize,
    key_frames: usize
}

fn main() {
    todo!();
    /*let config = Config{
        slice_size: 16537,
        num_bands: 33,
        rec_frames: 441000,
        pair_frames: 132300,
        key_len: 512
        
    };
    
    let mut args = env::args();
    let _ = args.next();
    let mode = args.next();
    let addr =args.next().expect("arg missing");
    let dev_name = args.next().expect("arg missing");
    for i in 1..10{
        match mode{
            "r" => rec_pair(&addr, &dev_name, &config, |x, y| rec::record(x,y), ),
            "s" => try_pair(&addr, &dev_name, &config),
            _ => panic!("first argument must either be r or s")
        }
    }*/
}
fn distance(first: &Vec<u8>, second: &Vec<u8>)->usize{
    let mut sum =0;
    for i in 0..first.len(){
        if first[i]!=second[i] {sum = sum+1;}
    }
    sum
}
fn align(first: &Vec<u8>, second: &Vec<u8>)->usize {
    let mut offset = (0, 0isize);
    let max_len = if first.len() < second.len() { first.len() } else {second.len()};
    for i in 0..max_len{
        let mut cross_corr = 0isize;
        for j in 0..(max_len-i){
            cross_corr=cross_corr+(first[j] as isize*second[i+j] as isize);
        }
        cross_corr=cross_corr/(max_len-i) as isize;
        if cross_corr> offset.1 {
            offset = (i,cross_corr)
        }
       
    }
    offset.0
}
fn hanning_window(input: &[i16], window_len: usize)->Vec<f32>{
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
        let val = &line.expect("Error reading file");
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
        buffer.push(Complex32::new(*data.get(i).expect("Error building FFT input buffer"), 0.0));
        i=i+1;
    }
    plan.process(&mut buffer);
    buffer.to_vec()
}

fn fingerprint(data: &[i16], config: &Config)->Vec<bool>{
    //Probably should end up with this back inside the call to fourier, and just replace calls to data.len() with transformed.len()
    if {data.len()%config.slice_size !=0} {
        /*Possibly return an error instead of truncating for immutable borrow*/
        //data.truncate(data.len()-data.len()%config.slice_size);
        panic!("Dimension error");
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
                true
            }else{
               false
            });
            j = j+1;
        };
        i=i+1;  
    };
 //  println!("Fingerprint built");
    to_ret
    
}

fn write_txt(filename: &str, buf: Vec<u8>)->(){
    //let mut target = File::create(filename).unwrap();
    let mut target = OpenOptions::new().append(true).create(true).open(filename).expect("Error opening file to write");
     for i in 0..buf.len() {
         target.write_all(buf[i].to_string().as_bytes()).expect("Error writing file");
         //Line breaks appear to be undesirable
         //target.write_all("\n".as_bytes()).unwrap();
         
     }
 }
 

 //Network logic starts here

fn try_pair<F,G>(addr: &str, dev_name: &str,config: &Config, rec_func:F, fingerprint_func:G)->Result<Vec<u8>>
where F: Fn(&str, usize)->Vec<u8>, G: Fn(Vec<u8>, Vec<u8>)->Vec<u8>{
    let mut stream= TcpStream::connect(addr)?;
	stream.set_nonblocking(false)?;
    println!("Connected");
    let buf=[0; 1];
    stream.write(&buf)?;    
    let mut data = rec_func(dev_name, config.rec_frames);
    let mut buffer = Vec::with_capacity(config.pair_frames);
	buffer.resize(config.pair_frames, 0);
	let mut filled = 0;
    let buf = buffer.as_mut_slice();
	while filled < config.pair_frames*2 {
        let t =  stream.read(&mut buf[filled..config.pair_frames*2])?;
        filled = filled+t;
	}
    //let received_data: Vec<u8> = to_i16(buffer);//Vec::from_raw_parts(buffer.as_mut_ptr() as *mut i16, buffer.len()/2, buffer.len()/2);
    //let offset=align(&buffer, &data);
    //println!("Offset: {}", offset);
    //{data.drain(0..(offset+buffer.len()));}
    let mut fp = fingerprint_func(data, buffer);
    fp.resize(config.key_len, 0);   
    return Ok(fp);    
}
#[allow(unreachable_code)]
fn rec_pair<F,G>(addr: &str, dev_name: &str,config: &Config, rec_func:F, fingerprint_func:G)->Result<Vec<u8>>
where F: Fn(&str, usize)->Vec<u8>,
    G: Fn(Vec<u8>)->Vec<u8>{
    let listener = TcpListener::bind(addr)?;
    for stream in listener.incoming() {
        let mut stream = stream?;//.expect("Failed to unwrap stream in rec_pair");
        let mut buf= Vec::with_capacity(1);
        let mut buf = buf.as_mut_slice();
        loop{
            let t = stream.read(&mut buf)?;
            if t != 0{
                let mut pair_data = rec_func(dev_name, config.rec_frames);
                let fp_data = pair_data.split_off(config.pair_frames);
                //let to_send: Vec<u8> = to_u8(pair_data);//Vec::from_raw_parts(pair_data.as_mut_ptr() as *mut u8, pair_data.len()*2, pair_data.capacity()*2);
                stream.write(&pair_data)?;
                let mut res=fingerprint_func(fp_data);
                println!("Fingerprint has len: {}", res.len());
                res.resize(config.key_len, 0);
                //write_txt("./receiver_key.txt", res);
                return Ok(res);
            }
        }
        unreachable!();
    }
    unreachable!();
}
fn to_u8(input: Vec<i16>)->Vec<u8>{
	let mut to_ret:Vec<u8> = Vec::with_capacity(input.len()*2);
	for i in 0..input.len(){
		let temp = input[i].to_be_bytes();
		for j in 0..temp.len(){
			to_ret.push(temp[j]);
		}
	}
	to_ret
}
fn to_i16(input: &[u8])->Vec<i16>{
	let mut to_ret = Vec::with_capacity(input.len()/2);
	for chunk in input.chunks_exact(2) {
		to_ret.push(i16::from_be_bytes(chunk.try_into().unwrap()));
	}
	to_ret
}
fn from_bools(input: &[bool])->Vec<u8>{
    let mut to_ret = Vec::with_capacity(input.len()/8);
    for chunk in input.chunks(8){
        let mut el = 0;
        for i in 0..chunk.len(){
            el+=chunk[i] as u8;
            el = el << 1;
        }
        to_ret.push(el);
    }
    to_ret
}
fn from_usize(input: &[usize])->Vec<u8>{
    let mut to_return = Vec::with_capacity(input.len()*size_of::<usize>());
    for i in 0..input.len(){
        let temp = input[i].to_ne_bytes();
        for j in 0..temp.len(){
            to_return.push(temp[j]);
        }
    }
    to_return
}
fn check_hash(first: &[u8], second:&[u8])->bool{
    for i in 0..first.len(){
        if first[i] != second[i] {return false;}
    }
    return first.len()==second.len();
}
fn receive_message(data: Vec<u8>, mut received:Vec<u8>, config: &Config)->Vec<usize>{
    let offset = align(&data, &received)/2;
	let data = to_i16(data.as_slice());
	let mut message = received.split_off(config.pair_frames);
	let hash = message.split_off(message.len()-512);
	
	let mut test= Vec::with_capacity(message.len());
	let field = reed_solomon::GaloisField::new(10, 0b10000001001, 0b00000000011);
	let rs = reed_solomon::ReedSolomon{
		field: field,
		n: 512,
		k: 204
	};
	for i in 0..200 {
		let slice = if offset <44100/5 { &data[i*441..config.key_frames+i*441]}else{&data[offset-44100/5+i*441..config.key_frames+offset-44100/5+i*441]};
		let fp = from_bools(&fingerprint(slice, config));
		//let com
		for j in 0..message.len(){
			test[i] = message[i] as usize ^fp[i] as usize;
		}
		let mut test_poly = reed_solomon::Poly{coeffs: test};
		test_poly = rs.decode(test_poly).unwrap();
		let mut hasher = sha2::Sha512::new();
		hasher.update(from_usize(&test_poly.coeffs));
		let result = hasher.finalize();
		if check_hash(&result.as_slice(), &hash){
			return test_poly.coeffs;
		}
		test = test_poly.coeffs; 
	}
	return Vec::new();
}
fn send_message(message: Vec<usize>, mut recorded: Vec<u8>, config:&Config)->Vec<u8>{
    let key = from_bools(&fingerprint(&to_i16(&recorded.split_off(config.pair_frames)), config));
    let field = reed_solomon::GaloisField::new(10, 0b10000001001, 0b00000000011);
	let rs = reed_solomon::ReedSolomon{
		field: field,
		n: 512,
		k: 204
	};
    let mut mess_poly=from_usize(&rs.encode(reed_solomon::Poly{coeffs:message}).unwrap().coeffs);
    let mut hasher = sha2::Sha512::new();
    hasher.update(&mess_poly);
    let result = hasher.finalize();
    for i in 0..mess_poly.len(){
        mess_poly[i]=mess_poly[i]^key[i];
    }
    recorded.extend_from_slice(&mess_poly);
    recorded.extend_from_slice(&result.as_slice());
    recorded
    //todo!();
}