use std::{env, thread, time, mem, boxed};
use alsa::{Direction, ValueOr};
use alsa::pcm::{PCM, HwParams, Format, Access, State};
use hound;
use std::fs::File;
use std::io::Write
use std::fs::OpenOptions;
fn main() {
    let mut args = env::args();
    let _ = args.next(); //Throw away the filename
    let dev_name=args.next().expect("Insufficient arguments.  Must provide device name.");
   // let out_dev_name = args.next().expect("Insufficient arguments.  Must provide output device");
    let wav_name = args.next().expect("Insufficient arguments.  Must provide filename to write wav to");
    let text_name = args.next().expect("Insufficient args.");
    for i in 1..100{
   let buf = record(&dev_name, 573300);
    //write_wav(&wav_name, &buf);
    write_txt(&text_name, &buf);
    }
  // playback(&out_dev_name, buf);
}

fn record(dev_name: &str, frames: usize)->Vec<i16>{
    let pcm = PCM::new(dev_name, Direction::Capture, false).unwrap();
   let hwp = HwParams::any(&pcm).unwrap();
println!("Min channels: {}", hwp.get_channels_min().unwrap());
println!("Max channels: {}", hwp.get_channels_max().unwrap());
    hwp.set_channels(2).unwrap();
    hwp.set_rate(44100, ValueOr::Nearest).unwrap();
    hwp.set_format(Format::s16()).unwrap();
    hwp.set_access(Access::RWInterleaved).unwrap();
    pcm.hw_params(&hwp).unwrap();
    let io = pcm.io_i16().unwrap();
    //Should probably separate setting up all this from the actual record
	//enough space for 6.5 seconds of recording at 44100Hz with 2 channels
    let mut buf = Vec<i16>::with_capacity(frames).as_slice();//[0i16; 573300];
    let reads=io.readi(&mut buf).unwrap();
    println!("Read {} frames", reads);
//    println!("Received following data: {:#?}", buf);
    boxed::Box::new(buf).to_vec()
}
fn playback(dev_name: &str, buf: &Vec<i16>)->(){
    let writePCM=PCM::new(dev_name, Direction::Playback, false).unwrap();

    let ohwp = HwParams::any(&writePCM).unwrap();
    ohwp.set_channels(1).unwrap();
    ohwp.set_rate(44100, ValueOr::Nearest).unwrap();
    ohwp.set_format(Format::s16()).unwrap();
    ohwp.set_access(Access::RWInterleaved).unwrap();
    writePCM.hw_params(&ohwp).unwrap();
    let oio = writePCM.io_i16().unwrap();
    oio.writei(&buf.as_slice());
}
fn write_wav(filename: &str, buf: &Vec<i16>)->(){
    let specs = hound::WavSpec{
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int
    };
    let mut writer=hound::WavWriter::create(filename, specs).unwrap();
	let mut iwriter = writer.get_i16_writer(buf.len()as u32);
	for n in buf.iter() {
		iwriter.write_sample(*n);
	}
//    buf.iter().for_each(|x| writer.write_sample(x)).;
    iwriter.flush().unwrap();

}
fn write_txt(filename: &str, buf: &Vec<i16>)->(){
   //let mut target = File::create(filename).unwrap();
   let mut target = OpenOptions::new().append(true).create(true).open(filename).unwrap();
    for i in 0..buf.len() {
        if{i%2==0}{
        target.write_all(buf[i].to_string().as_bytes());
        //Line breaks appear to be undesirable
        //target.write_all("\n".as_bytes());
        }
    }
}
