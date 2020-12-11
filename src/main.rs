#![feature(asm)]
use std::{net::TcpStream, io::{Read,Write}, str::from_utf8};


// define static strings here
static amd_bochs: &str = "AMD Athlon(tm) processor";
static intel_bochs: &str = "              Intel(R) Pentium(R) 4 CPU        ";
static qemu: &str = "QEMU Virtual CPU";

struct CheckVM{
    we_good: bool,
}

impl CheckVM {
    fn _get_cpu_brand(&mut self, buffer: &mut [u8], offset: usize, level: u32){
        let mut eax: u32 = level;
        let mut ebx: u32;
        let mut ecx: u32;
        let mut edx: u32;
        
        // run CPUID with special flags
        unsafe{asm!{"mov eax, ebx", in("ebx") eax}};
        unsafe{asm!{"cpuid", inout("rax") eax, out("ebx") ebx, 
                                            out("ecx") ecx, out("edx") edx}};
        
        // convert to u8 bytes
        let eax_bytes = eax.to_be_bytes();
        let ebx_bytes = ebx.to_be_bytes();
        let ecx_bytes = ecx.to_be_bytes();
        let edx_bytes = edx.to_be_bytes(); 

        
        //https://github.com/a0rtega/pafish/blob/master/pafish/bochs.c

        // write the data to the stuff
        buffer[0+offset] = eax_bytes[3];
        buffer[1+offset] = eax_bytes[2];
        buffer[2+offset] = eax_bytes[1];
        buffer[3+offset] = eax_bytes[0];

        buffer[4+offset] = ebx_bytes[3];
        buffer[5+offset] = ebx_bytes[2];
        buffer[6+offset] = ebx_bytes[1];
        buffer[7+offset] = ebx_bytes[0];

        buffer[8+offset] = ecx_bytes[3];
        buffer[9+offset] = ecx_bytes[2];
        buffer[10+offset] = ecx_bytes[1];
        buffer[11+offset] = ecx_bytes[0];

        buffer[12+offset] = edx_bytes[3];
        buffer[13+offset] = edx_bytes[2];
        buffer[14+offset] = edx_bytes[1];
        buffer[15+offset] = edx_bytes[0];

        // terminate the string
        buffer[16] = 0;

    }
    
    fn get_cpu_brand(&mut self, cpu_brand: &mut [u8]){
        let mut eax: u32;

        unsafe{asm!{"mov eax, 0x80000000"}};
        unsafe{asm!{"cpuid"}};
        unsafe{asm!{"cmp eax, 0x80000004"}};
        unsafe{asm!{"xor eax, eax"}};
        unsafe{asm!{"setge al", out("eax") eax}};

        if eax != 0 {
            self._get_cpu_brand(cpu_brand, 0usize, 0x80000002);
            self._get_cpu_brand(cpu_brand, 16usize, 0x80000003);
            self._get_cpu_brand(cpu_brand, 32usize, 0x80000004);
            // zero the string
            cpu_brand[48] = 0;
        }
    
    }

    fn get_cpu_vendor(&mut self, cpu_vendor: &mut [u8]){
        let mut ebx: u32;
        let mut ecx: u32;
        let mut edx: u32;

        // so this is actually the part that doesnt work...
        unsafe{asm!{"xor eax, eax"}};
        //unsafe{asm!{"xor rbx, rbx"}};
        //unsafe{asm!{"xor rcx, rcx"}};
        //unsafe{asm!{"xor rdx, rdx"}};
        unsafe{asm!{"cpuid"}};
        unsafe{asm!{"nop", out("ebx") ebx, out("ecx") ecx, out("edx") edx}};

        
        // convert to u8 bytes
        let ebx_bytes = ebx.to_be_bytes();
        let ecx_bytes = ecx.to_be_bytes();
        let edx_bytes = edx.to_be_bytes(); 

        
        //https://github.com/a0rtega/pafish/blob/master/pafish/bochs.c

        // write the data to the stuff
        cpu_vendor[0] = ebx_bytes[3];
        cpu_vendor[1] = ebx_bytes[2];
        cpu_vendor[2] = ebx_bytes[1];
        cpu_vendor[3] = ebx_bytes[0];

        cpu_vendor[4] = edx_bytes[3];
        cpu_vendor[5] = edx_bytes[2];
        cpu_vendor[6] = edx_bytes[1];
        cpu_vendor[7] = edx_bytes[0];

        cpu_vendor[8] = ecx_bytes[3];
        cpu_vendor[8] = ecx_bytes[2];
        cpu_vendor[10] = ecx_bytes[1];
        cpu_vendor[11] = ecx_bytes[0];

        // terminate the string
        cpu_vendor[12] = 0;
        
    }

    fn check_bochs_amd1(&mut self) -> bool {
        let mut cpu_brand = [0 as u8; 49];

        self.get_cpu_brand(&mut cpu_brand);
        let vendor_str = from_utf8(&cpu_brand).unwrap();

        if vendor_str.eq(amd_bochs){
            return true;
        }

        false
    }

    // check secondary AMD problem
    fn check_bochs_amd2(&mut self) -> bool {
        let mut dat: i32;

        unsafe{asm!("xor eax, eax;")}; // zero out eax
        unsafe{asm!("cpuid;")}; // CPUID
        unsafe{asm!("cmp ecx, 0x444d4163;")}; // AMD CPU?
        unsafe{asm!("jne b2not_detected;")};
        unsafe{asm!("mov eax, 0x8fffffff;")}; // magic crap
        unsafe{asm!("cpuid;")};
        unsafe{asm!("jecxz b2detected;")};
        unsafe{asm!("b2not_detected: xor ebx, ebx; jmp b2exit;")};
        unsafe{asm!("b2detected: mov ebx, 0x1;")};
        unsafe{asm!("b2exit: nop", out("eax") dat)};
        
        dat == 1
    }

    fn check_bochs_intel(&mut self) -> bool {
        let mut cpu_brand = [0 as u8; 49];

        self.get_cpu_brand(&mut cpu_brand);
        let vendor_str = from_utf8(&cpu_brand).unwrap();

        if vendor_str.eq(intel_bochs){
            return true;
        }

        false
    }

    fn check_qemu_cpu(&mut self) -> bool{
        let mut cpu_brand = [0 as u8; 49];

        self.get_cpu_brand(&mut cpu_brand);
        let vendor_str = from_utf8(&cpu_brand).unwrap();

        if vendor_str.eq(intel_bochs){
            return true;
        }
        
        false
    }

    fn check_vm(&mut self) -> bool{
        let mut is_ok = false;
        is_ok |= self.check_bochs_amd1();
        is_ok |= self.check_bochs_amd2();
        is_ok |= self.check_bochs_intel();
        is_ok |= self.check_qemu_cpu();

        is_ok
    }

}



struct SneakiNet{
    id: u32, 
} 

impl SneakiNet {
    fn initialize(&mut self, target: &str){
        // attept to connect to the target 
        match TcpStream::connect(target) {
            Ok(mut stream) => {
                // connected
                println!("[INFO] Connected...");
                
                // write test message
                let msg = b"hola";
                stream.write(msg).unwrap();

                println!("[INFO] Message sent, awaiting reply...");

                let mut data = [0 as u8; 4];
                match stream.read_exact(&mut data){
                    Ok(_) => {
                        // woo our stuff matched :)
                        if &data == msg {
                            println!("[INFO] Completed message cycle");                        }
                    },
                    Err(e) => {
                        println!("[ERR] Failed: {}", e);
                    }
                }
            },
            Err(e) => {
                println!("[ERR] Failed: {}", e);
            } 
        }
        println!("[INFO] Terminated");
    }

    fn check_vm(&mut self) -> bool {
        let mut cvm = CheckVM{ we_good: false };
        //let mut cpu_brand = [0 as u8; 49];

        //cvm.get_cpu_brand(&mut cpu_brand);

        //for bit in cpu_brand.iter() {
        //    print!("{} ", bit);
        //}


        //let vendor_str = from_utf8(&cpu_brand).unwrap();

        //println!("CPU STRING: {}", vendor_str);

        cvm.check_vm()
    }

}




fn main() {
    let id = 1;
    let mut t = SneakiNet{ id };
    if t.check_vm(){
        println!("WE IN A MF VM LOL");
    } else {
        println!("We good :)");
    }
}
