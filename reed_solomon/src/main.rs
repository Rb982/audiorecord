#![allow(dead_code)]
/*
    Note - good chance that all the results should instead by Options
    Currently everything is using usize as the numeric type; there's basically no compelling reason for that, and probably the underlying numeric type should be u8
    Switch to u8 would allow us to type-level guarantee never receiving a number outside of 0-255, so for GF 2^8 we could omit all the error checks
    
*/
fn main(){}
struct GaloisField{
    gf_log: Vec<usize>,
    gf_ilog: Vec<usize> 
}
impl GaloisField{
    fn new(pow: usize, prim_poly: usize)->Self{
       
        let max: usize = 1<<pow;
        let mut gf_log = vec![0usize; max];
        let mut gf_ilog=vec![0usize; max];
        let mut b = 1;
        for log in 0..max{
            gf_log[b] = log;
            gf_ilog[log]=b;
            b = b<<1;
            if b&max!=0{
                b = b^prim_poly;
            }
        }
        GaloisField{
            gf_log: gf_log,
            gf_ilog: gf_ilog
        }
    }
    fn sum(&self, x: usize, y: usize)->Result<usize, &str>{
        if x>self.gf_log.len() || y>self.gf_log.len(){
            return Err("args out of bounds");

        }
         Ok(x^y)
    }
    fn mult(&self, x: usize, y: usize)->Result<usize, &str>{
        if x>self.gf_log.len() || y>self.gf_log.len(){
            return Err("args out of bounds");

        }
         Ok(self.gf_ilog[(self.gf_log[x]+self.gf_log[y]) % self.gf_ilog.len()])
    }
    fn div(&self, x: usize, y: usize)->Result<usize, &str>{
        if x>self.gf_log.len() || y>self.gf_log.len(){
            return Err("args out of bounds");

        }
         Ok(self.gf_ilog[(self.gf_log[x]-self.gf_log[y]+self.gf_ilog.len()) % self.gf_ilog.len()])
    }
    fn inverse(&self, x:usize)->Option<usize>{
        if x==0{
            return None
        }
        for i in 1..self.gf_log.len(){
            if self.mult(x, i)== Ok(1){
                Some(i);
            }
        }
        None

    }
    fn eval_poly_at(&self, f:&Poly, x: usize)->Result<usize, &str>{
        let mut to_ret = f.coeffs[0];
        let mut x_curr = x;
        for i in 1..f.coeffs.len(){
            to_ret = self.sum(to_ret,self.mult(f.coeffs[i],x_curr)?)?;
            x_curr = self.mult(x_curr, x)?;
        }
        Ok(to_ret)
    }
    fn sum_poly(&self, first: &Poly, second: &Poly)->Result<Poly, &str>{
        let (min, max, longest)= if first.deg()>second.deg(){
            (second.deg()+1, first.deg()+1, first)
        }else{
            (first.deg()+1, second.deg()+1, second)
        };
        let mut new_coeffs = Vec::with_capacity(max);
        for i in 0..min{
            new_coeffs.push(self.sum(first.coeffs[i], second.coeffs[i])?);
        }
        for j in min..max{
            new_coeffs.push(longest.coeffs[j]);
        }
        Ok(Poly{coeffs: new_coeffs})
        //todo!();
    }
    fn mult_poly(&self, first: &Poly, second: &Poly)->Result<Poly, &str>{
        let f_deg = first.deg();
        let s_deg = second.deg();
        let mut new_coeffs=Vec::with_capacity(f_deg+s_deg+1);
        for i in 0..f_deg+1{
            for j in 0..s_deg+1{
                if new_coeffs.len()<=i+j {
                    new_coeffs.push(self.sum(first.coeffs[i], second.coeffs[j])?);
                }else{
                    new_coeffs[i+j]=self.sum(new_coeffs[i+j], self.sum(first.coeffs[i], second.coeffs[j])?)?;
                }
            }
        }
        Ok(Poly{coeffs: new_coeffs})
    }
    fn div_poly(&self, dividend: &Poly, divisor: &Poly)->Result<(Poly, Poly), &str>{
        //todo!();
        let mut result  = vec![0; divisor.deg()];
        let mut remainder = Poly{coeffs: dividend.coeffs.clone()};
        while remainder.deg() >= divisor.deg() {
            let (coeff, deg) = (self.div(remainder.coeffs[remainder.deg()], divisor.coeffs[divisor.deg()])?, remainder.deg()-divisor.deg());
            remainder = self.sum_poly(
                &remainder,
                &self.mult_poly(divisor, &Poly::mononomial(coeff, deg))?
            )?;
            result[deg]=coeff;
        }
        Ok((Poly{coeffs: result}, remainder))
    }
}

struct Poly{
    coeffs: Vec<usize>,
   // field: &'a GaloisField
}

impl Poly{
    fn deg(&self)->usize{
        for i in 1..self.coeffs.len()+1{
            if self.coeffs[self.coeffs.len()-i]!=0 {
                return self.coeffs.len()-i
            }
        }
        return 0;
    }

    fn mononomial(coeff: usize, deg: usize)->Self{
        let mut coeffs = vec![0; deg+1];
        coeffs[deg] = coeff;
        Poly{coeffs}
    }
    //Equivalent to multiplying by X^count
    fn shift(mut self, count: usize)->Self{
        
        let mut prepend = vec![0usize; count];
        prepend.append(&mut self.coeffs);
        self.coeffs=prepend;
        self
    }

}


struct ReedSolomon{
    n: usize,
    k: usize,
    generator: Poly,
    field: GaloisField
}

impl ReedSolomon{
    fn encode(&self, mut message: Poly)->Result<Poly, &str>{
        message = message.shift(self.n-self.k);
        let (_, ck) = self.field.div_poly(&message, &self.generator)?;
        self.field.sum_poly(&message, &ck)
    }
}