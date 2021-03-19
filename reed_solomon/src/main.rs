#![allow(dead_code)]

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
        if x == 0{
            return Ok(0);
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
    fn pow(&self, x:usize, n:usize)->Result<usize, &'static str>{
        let mut to_ret = 1;
        for _i in 0..n{
            to_ret = self.mult(to_ret, x)?;
        }
        Ok(to_ret)
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
    fn roots(&self, f:&Poly)->Result<Vec<usize>, &'static str>{
        let deg = f.deg();
        let mut to_ret = Vec::with_capacity(deg);
        //let mut alpha = self.generator;
        for i in 0..self.gf_log.len(){
            if self.eval_poly_at(f, i)?==0 {
                to_ret.push(i);  
            }
            if to_ret.len()==deg {return Ok(to_ret);
            }
           // alpha = self.mult(alpha, self.generator)?;
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
    fn rref(&self, mut mat: Vec<Vec<usize>>)->Result<Vec<Vec<usize>>, &'static str>{
        let mut leading_coeff=0;
        for i in 0..mat.len(){
            while mat[i][leading_coeff]==0 {
               for j in i+1..mat.len() {
                    if mat[j][i] !=0{
                        mat.as_mut_slice().swap(i, j);
                        break;
                    }
                }
                if mat[i][leading_coeff]==0 {
                    leading_coeff+=1;
                    if leading_coeff==mat.len() {return Ok(mat);}
                }
            }
            let temp = mat[i][leading_coeff];
            for j in leading_coeff..mat[i].len(){
                mat[i][j] = self.div(mat[i][j], temp)?;
            }
            for j in 0..mat.len(){
                if j !=i{
                let mult_factor = mat[j][leading_coeff];
                for k in leading_coeff..mat[j].len(){
                    mat[j][k]=self.sum(mat[j][k], self.mult(mult_factor, mat[i][k])?)?;
                }
            }
            }
            leading_coeff+=1;
            if leading_coeff==mat.len() {return Ok(mat);}
        }
        Ok(mat)
    }
}
#[derive(Clone, Debug)]
struct Poly{
    coeffs: Vec<usize>
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
           to_ret.push(self.field.eval_poly_at(received, curr).unwrap());
           curr=self.field.mult(curr, self.field.generator).unwrap();
        }
        to_ret
    }
    fn berlekamp(&self, syndrome_components: &Vec<usize>)->Poly{
        let mut d = Vec::with_capacity(self.n-self.k);
        d.push(syndrome_components[0]);
        let mut sigma: Vec<Poly> = Vec::with_capacity(self.n-self.k);
        
        sigma.push(Poly::mononomial(1,0));
        let mut h = Vec::with_capacity(self.n-self.k);
        h.push(0);
        for i in 0..(self.n-self.k){
            //println!("i:{}, d:{}, h:{}, sigma:{:#?}", i, d[i], h[i], sigma[i]);   
	        if d[i] == 0 {
                      
		        sigma.push(sigma[i].clone());
		        h.push(h[i]);
	        }else{
                //(-1,-1) is a legal value for rho, but forces a bunch of converts between usize and isize, so we set rho to (0,0) and check special cases instead
                //Relevant special cases are d[rho.0]=0, in which case (-1,-1) is largest satisfying value, and i=0, in which case rho is always (-1,-1) if calculated
		        let mut rho = (0,0);
		        for j in 0..i{
			        if d[j] !=0 && j-h[j] >= rho.1 {
				        rho = (j, j-h[j])
			        }
		        }
		        let temp = i-rho.1;
		        h.push(if temp > h[i] {temp} else {h[i]});
		        let d_rho_inverse = if d[rho.0]== 0 || i==0 {1} else {self.field.mult_inverse(d[rho.0]).unwrap()};
                let unit = Poly::mononomial(1,0);
		        let sigma_rho = if d[rho.0] == 0 || i ==0 {&unit} else {&sigma[rho.0]};
                let x_pow = if d[rho.0]==0 || i == 0 {i+1} else {i-rho.0};
		        let sigma_next = self.field.sum_poly(&sigma[i], &self.field.mult_poly(&Poly::mononomial(self.field.mult(d[i], d_rho_inverse).unwrap(), x_pow), &sigma_rho).unwrap()).unwrap();
                sigma.push(sigma_next);
            }
            if i!=(self.n-self.k-1){
                let mut d_next=syndrome_components[i+1];
                //println!("i: {}, d_next: {}", i, d_next);
                for j in 1..sigma[i+1].coeffs.len(){
                    d_next=self.field.sum(d_next, self.field.mult(sigma[i+1].coeffs[j], syndrome_components[(i+1)-j]).unwrap()).unwrap();
                    //println!("j: {}, d_next: {}", j, d_next);
                }
                d.push(d_next);
            }
        }
       
        sigma.pop().unwrap()
        //sigma[sigma.len()-1]
    }
    fn error_locs(&self, elp_r:&Poly)->Result<Vec<usize>, &'static str>{
        //println!("ELP: {:#?}", elp_r);
        let mut to_ret = self.field.roots(elp_r)?;
        for i in 0..to_ret.len() {
            if to_ret[i]>self.field.gf_log.len() {
                return Err("args out of bounds");
            }
            to_ret[i] = self.field.gf_log[to_ret[i]];
        } 
        Ok(to_ret)
    }
    fn error_vals(&self, elc: &Vec<usize>, synd: &Vec<usize>)->Result<Vec<usize>, &'static str>{
        
            let mut mat = Vec::with_capacity(elc.len());
            let mut first_row = elc.clone();
            for i in 0..first_row.len(){
                first_row[i] = self.field.pow(self.field.generator, first_row[i]).unwrap();
            }
            first_row.push(synd[0]);
            mat.push(first_row);
            for i in 1..elc.len(){
                let mut row = mat[i-1].clone();
                for j in 0..row.len()-1{
                    row[j]=self.field.mult(row[j], mat[0][j])?;
                }
                let last = row.len()-1;
                row[last] = synd[i];
                mat.push(row);
            }
           // println!("mat: {:#?}", mat);
            mat = self.field.rref(mat)?;
           // println!("mat: {:#?}", mat);
            let mut to_ret = Vec::with_capacity(mat.len());
            for i in 0..mat.len(){
                to_ret.push(mat[i][mat[i].len()-1]);
            }
            Ok(to_ret)
        

    }
    fn decode(&self, received: Poly)->Result<Poly, &'static str>{
        /*
            1. Syndrome components
            2. Use syndrome components to get error locator poly from Berlekamp
            3. Get error locations and values from 1,2.
            4. Construct E(X)'
            5. Decoded = received+E(X)

        */
        let syndrome_components = self.syndrome_components(&received);
        //println!("Syndrome components: {:#?}", syndrome_components);
        let mut err_loc_poly_r = self.berlekamp(&syndrome_components);
       // println!("ELPR{:#?}", err_loc_poly_r);
        if err_loc_poly_r.deg()==0 {
            return Ok(received);
        }
        err_loc_poly_r.coeffs.as_mut_slice().reverse();
        let err_locs = self.error_locs(&err_loc_poly_r)?;
       // println!("Error locations: {:#?}", err_locs);
        let error_vals = self.error_vals(&err_locs, &syndrome_components)?;
       // println!("Error values: {:#?}", error_vals);
        let mut error_poly = Poly::mononomial(0,0);
        for i in 0..err_locs.len(){
            error_poly = self.field.sum_poly(&error_poly, &Poly::mononomial(error_vals[i], err_locs[i]))?
        }
        //println!("Error poly: {:#?}", error_poly);
        let to_ret = self.field.sum_poly(&received, &error_poly)?;
        //{to_ret.coeffs.drain(0..(self.n-self.k));}
        Ok(to_ret)
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
                    Ok(t) => assert!(t==i && j != 0, "Division is inverse multiplication: i:{},j:{},t{}",i,j,t),
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
    
    #[test]
    fn test_generator()->Result<(), &'static str>{
        let field = GaloisField::new(4, 0b10011,0b0010);
        let expected=vec![field.pow(field.generator, 6)?,field.pow(field.generator, 9)?,field.pow(field.generator, 6)?,field.pow(field.generator, 4)?,field.pow(field.generator, 14)?,field.pow(field.generator, 10)?,1];
        let rs = ReedSolomon{
            n: 15,
            k: 9,
            field: field
        };
        let generator=rs.generator_poly();
        for i in 0..generator.coeffs.len(){
            assert!(generator.coeffs[i]==expected[i]);
        }
        Ok(())
    }
    #[test]
    fn test_syndrome()->Result<(), &'static str>{
        let field = GaloisField::new(4, 0b10011,0b0010);
        let received = Poly{coeffs: vec![field.pow(field.generator,12)?,field.pow(field.generator,8)?, field.pow(field.generator,3)?, 
        field.pow(field.generator,4)?, field.pow(field.generator,10)?, field.pow(field.generator,8)?, 0, field.pow(field.generator,11)?, 1]};
        let correct_syndromes = vec![1,1,field.pow(field.generator,5)?,1,0,field.pow(field.generator,10)?];
        let rs = ReedSolomon{n:15, k:9, field: field};
        let synd= rs.syndrome_components(&received);
        for i in 0..synd.len(){
            assert!(synd.contains(&correct_syndromes[i]));
            assert!(correct_syndromes.contains(&synd[i]));
        }
        Ok(())
    }
    #[test]
    fn test_berlekamp()->Result<(), &'static str>{
        let field = GaloisField::new(4, 0b10011,0b0010);
        let received = Poly{coeffs:vec![field.pow(field.generator,12)?,field.pow(field.generator,8)?, field.pow(field.generator,3)?, 
        field.pow(field.generator,4)?, field.pow(field.generator,10)?, field.pow(field.generator,8)?, 0, field.pow(field.generator,11)?, 1]};
        let correct_sigma = vec![1, 1, field.pow(field.generator, 10)?];
        let rs = ReedSolomon{
            n:15,
            k:9,
            field:field
        };
        let result = rs.berlekamp(&rs.syndrome_components(&received));
        for i in 0..result.coeffs.len(){
            assert!(result.coeffs[i]==correct_sigma[i], "result: {:#?}", result);
        }
        assert!(result.coeffs.len()==3);
        Ok(())
    }
    #[test]
    fn test_error_correction()->Result<(),&'static str>{
        let field = GaloisField::new(4, 0b10011,0b0010);
        let received = Poly{coeffs:vec![field.pow(field.generator,12)?,field.pow(field.generator,8)?, field.pow(field.generator,3)?, 
        field.pow(field.generator,4)?, field.pow(field.generator,10)?, field.pow(field.generator,8)?, 0, field.pow(field.generator,11)?, 1]};
        let correct = vec![field.pow(field.generator,12)?,field.pow(field.generator,8)?, field.pow(field.generator,3)?^1, 
        field.pow(field.generator,4)?, field.pow(field.generator,10)?, field.pow(field.generator,8)?, 0, field.pow(field.generator,11)?, 1^1];
        let rs = ReedSolomon{n:15, k:9, field:field};
        



        let result = rs.decode(received)?;
        for i in 0..result.coeffs.len(){
            assert!(result.coeffs[i]==correct[i], "received: {:#?}, expected: {:#?}", result, correct);
        }
        assert!(result.coeffs.len()==correct.len());
        Ok(())
    }
}