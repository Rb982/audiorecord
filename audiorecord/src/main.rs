use std::env;
use alsa::{Direction, ValueOr};
use alsa::pcm::{PCM, HwParams, Format, Access, State};

fn main() {
    let args = env::args();
    let _ = args.next(); //Throw away the filename
    let devName=args.next().expect("Insufficient arguments.  Must provide device name.");
    //Threshold will need to become an i16 eventually, but not necessary yet.
    let threshold = args.next().expect("Insufficient arguments.  Must provide threshold");
    //Bool is for the nonblock property
   let pcm = PCM::new(&devName, Direction::Capture, false);
}
