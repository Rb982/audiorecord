#![allow(dead_code)]
/*
    Note - good chance that all the results should instead by Options
    Currently everything is using usize as the numeric type; there's basically no compelling reason for that, and probably the underlying numeric type should be u8
    Switch to u8 would allow us to type-level guarantee never receiving a number outside of 0-255, so for GF 2^8 we could omit all the error checks
    
*/
fn main(){}
struct GaloisField{
    gf_log: Vec<usize>,
    gf_ilog: Vec<usize>,
    generator: usize
}
impl GaloisField{
    fn new(pow: usize, prim_poly: usize, generator: usize)->Self{
       
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
       //gf_ilog.truncate(gf_ilog.len()-1);
        GaloisField{
            gf_log: gf_log,
            gf_ilog: gf_ilog,
            generator: generator
        }
    }
    fn sum(&self, x: usize, y: usize)->Result<usize, &'static str>{
        if x>self.gf_log.len() || y>self.gf_log.len(){
            return Err("args out of bounds");

        }
         Ok(x^y)
    }
    fn mult(&self, x: usize, y: usize)->Result<usize, &'static str>{
        if x>self.gf_log.len() || y>self.gf_log.len(){
            return Err("args out of bounds");

        }
        if x == 0 || y == 0 {
            return Ok(0);
        }
         Ok(self.gf_ilog[(self.gf_log[x]+self.gf_log[y]) % (self.gf_ilog.len()-1)])
    }
    fn div(&self, x: usize, y: usize)->Result<usize, &'static str>{
        if x>self.gf_log.len() || y>self.gf_log.len(){
            return Err("args out of bounds");

        }
         Ok(self.gf_ilog[((self.gf_ilog.len()-1)+self.gf_log[x]-self.gf_log[y]) % (self.gf_ilog.len()-1)])
    }
    fn mult_inverse(&self, x:usize)->Option<usize>{
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
    field: GaloisField
}

impl ReedSolomon{
    //Only way this can fail is if the original field had an invalid generator polynomial
    //Since in such a case everything we do is nonsense, we can assume that doesn't happen
    //Consequently, we can return a poly instead of a Result<Poly>
    fn generator_poly(&self)->Poly{
      
        let mut result = Poly::mononomial(1, 0);
        let mut temp=Poly{coeffs: vec![self.field.generator, 1]};
        for _i in 0..(self.n-self.k){
            if let Ok(t)=self.field.mult_poly(&result, &temp){
                result = t;
            }
            if let Ok(t) = self.field.mult(self.field.generator, temp.coeffs[0]){
                temp.coeffs[0]=t;
            }
        }
        result
    }
    fn encode(&self, mut message: Poly)->Result<Poly, &str>{
        message = message.shift(self.n-self.k);
        let (_, ck) = self.field.div_poly(&message, &self.generator_poly())?;
        self.field.sum_poly(&message, &ck)
    }
    //As in generator_poly, no need to return a result because there's no way to get an error with a correctly built GF
    fn syndrome_components(&self, received: &Poly)->Vec<usize>{
        let mut to_ret = Vec::with_capacity(self.n-self.k);
        let mut curr = self.field.generator;
        for _i in 0..(self.n-self.k){
            if let Ok(t)=self.field.eval_poly_at(received, curr){
                to_ret.push(t);
            }
            if let Ok(t)=self.field.mult(curr, self.field.generator){
                curr=t;
            }
        }
        to_ret
    }

}






#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_setup(){
        let prim_poly: usize = 0b100011101;
        let pow = 8;
        let generator = 0b10000011;
        let field = GaloisField::new(pow, prim_poly, generator);
        assert!(field.gf_log.len()==256);
        assert!(field.gf_ilog.len()==field.gf_log.len());
        for i in 0..field.gf_log.len(){
            for j in 0..field.gf_log.len(){
                if i!=j{
                    assert!(field.gf_log[i]!=field.gf_log[j], "Failure at {}, {}.  gf_log: {:#?}", i, j, field.gf_log);
                    assert!(field.gf_ilog[i]!=field.gf_ilog[j]||field.gf_ilog[i]==1);
                }
            }
        }

    }
    #[test]
    fn test_addition()->Result<(), &'static str>{
        let prim_poly: usize = 0b100011101;
        let pow = 8;
        let generator = 0b10000011;
        let field = GaloisField::new(pow, prim_poly, generator);
        for i in 0..255{
            for j in 0..255{
                let sum = field.sum(i, j)?;
                let should_be_i = field.sum(sum, j)?;
                assert!(i==should_be_i, "Addition is own inverse");
                assert!(sum<256, "Field is closed under addition");
                if(j==0){
                    assert!(sum==i, "Additive identity is an identity");
                }
                assert!(sum==i^j, "Addition is xor");
            }
        }
        Ok(())
    }
    #[test]
    fn test_multiplication()->Result<(), &'static str>{
        let prim_poly: usize = 0b100011101;
        let pow = 8;
        let generator = 0b10000011;
        let field = GaloisField::new(pow, prim_poly, generator);
        for i in 0..255{
            let by_zero = field.mult(i, 0)?;
            let by_id= field.mult(i, 1)?;
            assert!(by_id == i, "Multiplying by 1 yields original value: (i, by_id)= {},{}", i, by_id);
            assert!(by_zero == 0, "Multiplying by zero yields zero");
            for j in 0..255{
                let i_j = field.mult(i, j)?;
                let j_i = field.mult(j, i)?;
                if(i!=0&&j!=0){
                let should_be_i = field.div(i_j, j)?;
                assert!(should_be_i==i, "Division is inverse multiplication");
                }
                assert!(i_j == j_i, "Multiplication commutes");
                assert!(i_j<256, "Field is closed under multiplication");
                for k in 0..255{
                    let i_jk = field.mult(i, field.sum(j, k)?)?;
                    let ij_ik=field.sum(field.mult(i, j)?, field.mult(i, k)?)?;
                    assert!(i_jk == ij_ik, "Multiplication distributes over addition");
                }
            }
        }
        match field.mult(1000, 2){
            Ok(_)=>panic!("Invalid multiplications should fail"),
            Err(_)=>()
        };
        Ok(())
    }
}