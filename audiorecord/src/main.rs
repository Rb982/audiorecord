use std::env;
use alsa::{Direction, ValueOr};
use alsa::pcm::{PCM, HwParams, Format, Access, State};

fn main() {
    /*let args = env::args();
    let _ = args.next(); //Throw away the filename
    let devName=args.next().expect("Insufficient arguments.  Must provide device name.");
    //Threshold will need to become an i16 eventually, but not necessary yet.
    let threshold = args.next().expect("Insufficient arguments.  Must provide threshold");
    //Bool is for the nonblock property*/
    let devName="";
    
   let pcm = PCM::new(&devName, Direction::Capture, false);
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
}
