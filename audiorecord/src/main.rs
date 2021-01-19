use std::{env, thread, time};
use alsa::{Direction, ValueOr};
use alsa::pcm::{PCM, HwParams, Format, Access, State};

fn main() {
    let args = env::args();
    let _ = args.next(); //Throw away the filename
    let dev_name=args.next().expect("Insufficient arguments.  Must provide device name.");
    //Threshold will need to become an i16 eventually, but not necessary yet.
    let out_dev_name = args.next().expect("Insufficient arguments.  Must provide output device");
    //Bool is for the nonblock property*/
    let dev_name="";
    let out_dev_name="";
   let pcm = PCM::new(&dev_name, Direction::Capture, false);
   let hwp = HwParams::any(&pcm).unwrap();
    hwp.set_channels(1).unwrap();
    hwp.set_rate(44100, ValueOr::Nearest).unwrap();
    hwp.set_format(Format::s16()).unwrap();
    hwp.set_access(Access::RWInterleaved).unwrap();
    pcm.hw_params(&hwp).unwrap();
    let io = pcm.io_i16().unwrap();


    let mut buf = [0i16; 1024];
    let reads=io.readi(&buf).unwrap();
    println!("Read {} frames", reads);
    println!("Received following data: {:#?}", buf);
    thread::sleep(time::Duration::from_millis(5000));

    let writePCM=PCM::new(&out_dev_name, Direction::Playback, false);

    let ohwp = HwParams::any(&writePCM).unwrap();
    ohwp.set_channels(1).unwrap();
    ohwp.set_rate(44100, ValueOr::Nearest).unwrap();
    ohwp.set_format(Format::s16()).unwrap();
    ohwp.set_access(Access::RWInterleaved).unwrap();
    pcm.hw_params(&ohwp).unwrap();
    let oio = writePCM.io_i16().unwrap();
    oio.writei(&buf);
}
