use std::{env, thread, time, mem, boxed};
use alsa::{Direction, ValueOr};
use alsa::pcm::{PCM, HwParams, Format, Access, State};
use hound;
fn main() {
    let mut args = env::args();
    let _ = args.next(); //Throw away the filename
    let dev_name=args.next().expect("Insufficient arguments.  Must provide device name.");
    let out_dev_name = args.next().expect("Insufficient arguments.  Must provide output device");
    let wav_name = args.next().expect("Insufficient arguments.  Must provide filename to write wav to");
   let buf = record(&dev_name);
   playback(&out_dev_name, buf);
}

fn record(dev_name: &str)->Box<[i16]>{
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


    let mut buf = [0i16; 1024];
    let reads=io.readi(&mut buf).unwrap();
    println!("Read {} frames", reads);
    println!("Received following data: {:#?}", buf);
    boxed::Box::new(buf).into_vec()
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
fn write_out(filename: &str, buf: Vec<i16>)->(){
    let specs = hound::WavSpec{
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int
    };
    let mut writer=hound::WavWriter::new(filename, specs).get_i16_writer(buf.len());
    buf.iter.for_each(|x| writer.write_sample(x));
    writer.flush().unwrap();

}
