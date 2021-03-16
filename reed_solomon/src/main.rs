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
        if y == 0 {
            return Err("Division by zero");
        }
         Ok(self.gf_ilog[((self.gf_ilog.len()-1)+self.gf_log[x]-self.gf_log[y]) % (self.gf_ilog.len()-1)])
    }
    fn mult_inverse(&self, x:usize)->Option<usize>{
        if x==0{
            return None
        }
        for i in 1..self.gf_log.len(){
            match self.mult(x, i) {
                Ok(1)=>return Some(i),
                _=>()
            };
        }
        None

    }
    fn eval_poly_at(&self, f:&Poly, x: usize)->Result<usize, &'static str>{
        let mut to_ret = f.coeffs[0];
        let mut x_curr = x;
        for i in 1..f.coeffs.len(){
            to_ret = self.sum(to_ret,self.mult(f.coeffs[i],x_curr)?)?;
            x_curr = self.mult(x_curr, x)?;
        }
        Ok(to_ret)
    }
    fn sum_poly(&self, first: &Poly, second: &Poly)->Result<Poly, &'static str>{
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
    fn mult_poly(&self, first: &Poly, second: &Poly)->Result<Poly, &'static str>{
        let f_deg = first.deg(); 
        let s_deg = second.deg(); 
        let mut new_coeffs=Vec::with_capacity(f_deg+s_deg+1);
        for i in 0..f_deg+1{
            for j in 0..s_deg+1{
                if new_coeffs.len()<=i+j {
                    new_coeffs.push(self.mult(first.coeffs[i], second.coeffs[j])?);
                }else{
                    new_coeffs[i+j]=self.sum(new_coeffs[i+j], self.mult(first.coeffs[i], second.coeffs[j])?)?;
                }
            }
        }
        Ok(Poly{coeffs: new_coeffs})
    }
    fn div_poly(&self, dividend: &Poly, divisor: &Poly)->Result<(Poly, Poly), &'static str>{
        //todo!();
        
        let mut result  = vec![0; dividend.deg() - divisor.deg()+1];
        if divisor.deg()==0 {
           for i in 0..result.len(){
            result[i]=self.div(dividend.coeffs[i], divisor.coeffs[0])?;
           }
           return Ok((Poly{coeffs:result}, Poly::mononomial(0,0)));
        }
        let mut remainder = Poly{coeffs: dividend.coeffs.clone()};
        //Think this while condition is wrong; notably, if divisor is of degree zero, it's an infinite loop
        //Not sure if that has to be special-cased or if it's a sign my logic is wrong
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
    fn encode(&self, mut message: Poly)->Result<Poly, &'static str>{
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
                if j==0 {
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
               
                match field.div(i_j, j){
                    Ok(t) => assert!(t==i && j != 0, "Division is inverse multiplication"),
                    Err(_) =>assert!(j==0, "Division fails only for division by zero or non-field elements")
                }
               // assert!(should_be_i==i, "Division is inverse multiplication");
               
                assert!(i_j == j_i, "Multiplication commutes");
                assert!(i_j<256, "Field is closed under multiplication");
                for k in 0..255{
                    let i_jk = field.mult(i, field.sum(j, k)?)?;
                    let ij_ik=field.sum(field.mult(i, j)?, field.mult(i, k)?)?;
                    assert!(i_jk == ij_ik, "Multiplication distributes over addition");
                }
            }
            if let Some(t) = field.mult_inverse(i){
                assert!(field.mult(i, t)?==1);
            }else{
                assert!(i==0);
            }
        }
        match field.mult(1000, 2){
            Ok(_)=>panic!("Invalid multiplications should fail"),
            Err(_)=>()
        };
        Ok(())
    }
    #[test]
    fn test_polynomial()->Result<(), &'static str>{
        let prim_poly: usize = 0b100011101;
        let pow = 8;
        let generator = 0b10000011;
        let field = GaloisField::new(pow, prim_poly, generator);
        let zero_p=Poly::mononomial(0,0);
        let one_p=Poly::mononomial(1,0);
        //Ideally, p_one, p_two, and p_three should probably be randomly generated
        let p_one = Poly{
            coeffs: vec![239,228,65,146]
        };
        let p_two = Poly{
            coeffs: vec![167,54, 253, 181, 41]
        };
        let p_three=Poly{
            coeffs:vec![145,183]
        };
        let mult_by_zero = field.mult_poly(&p_one, &zero_p)?;
        let mult_by_one = field.mult_poly(&p_one, &one_p)?;
        let add_zero = field.sum_poly(&p_one, &zero_p)?;
       
        let prod_of_poly = field.mult_poly(&p_one, &p_two)?;
        let sum_of_poly = field.sum_poly(&p_one, &p_two)?;
       
        let (should_be_p_one, should_be_zero) = field.div_poly(&prod_of_poly, &p_two)?;
        
        let should_be_p_two = field.sum_poly(&sum_of_poly, &p_one)?;
        let one_twothree = field.mult_poly(&p_one, &field.sum_poly(&p_two, &p_three)?)?;
        let onetwo_onethree = field.sum_poly(&field.mult_poly(&p_one, &p_two)?, &field.mult_poly(&p_one, &p_three)?)?;
        
        let (div_res, div_rem) = field.div_poly(&p_two, &p_three)?;
       
        let should_also_be_p_two = field.sum_poly(&field.mult_poly(&div_res, &p_three)?, &div_rem)?;
        let should_be_err = field.div_poly(&p_two, &zero_p);
        match should_be_err{
            Ok(_)=>panic!("Div by zero should yield an error"),
            Err(_)=>()
        };
        let (should_be_p_three, _) = field.div_poly(&p_three, &one_p)?;
        for i in 0..255{
            let p_one_val = field.eval_poly_at(&p_one, i)?;
            let p_two_val = field.eval_poly_at(&p_two, i)?;
            let zero_res = field.eval_poly_at(&mult_by_zero, i)?;
            assert!(zero_res==0, "Multiplication by zero yields zero, {}, {}", zero_res, i);
            assert!(field.eval_poly_at(&mult_by_one, i)?==p_one_val, "Multiplicative id is one");
            assert!(field.eval_poly_at(&add_zero, i)?==p_one_val, "0 is additive id");
            assert!(field.eval_poly_at(&prod_of_poly, i)? == field.mult(p_one_val, p_two_val)?, "eval of mult of poly is mult of eval of poly");
            assert!(field.eval_poly_at(&sum_of_poly, i)? == field.sum(p_one_val, p_two_val)?, "eval of sum of poly is sum of eval of poly");
            assert!(field.eval_poly_at(&should_be_p_one, i)? == p_one_val, "division is inverse mult");
            assert!(field.eval_poly_at(&should_be_zero, i)? == 0, "Remainder of even division is zero poly");
            assert!(field.eval_poly_at(&should_be_p_two, i)? == p_two_val, "Addition is own inverse");
            assert!(field.eval_poly_at(&one_twothree, i)? == field.eval_poly_at(&onetwo_onethree, i)?, "Multiplication distributes");
            assert!(field.eval_poly_at(&should_also_be_p_two, i)?==p_two_val, "result*divisor+remainder = original value");
            assert!(field.eval_poly_at(&should_be_p_three, i)?==field.eval_poly_at(&p_three, i)?, "Div by one yields original value");
        } 

       Ok(())
    }
}