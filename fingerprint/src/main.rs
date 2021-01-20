
fn main() {
   let test = vec![1,1,1,1,1];
   println!("{:#?}", hanning_window(&test));
}


fn hanning_window(input: &Vec<i16>)->Vec<f32>{
    let mut i = 0;
    let mut to_ret = Vec::with_capacity(input.len());
    while i<input.len() {
        let hann=0.5f32 * (1f32-(2f32*std::f32::consts::PI*i as f32/input.len() as f32).cos());
        to_ret.push(hann * *input.get(i).unwrap() as f32);
        i=i+1;
    }
    to_ret
}