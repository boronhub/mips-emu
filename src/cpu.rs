use super::memory::*;


// FMT

// physical addr
pub const BFM_BASE: u32 = 0x1FC00000;
pub const BFM_SIZE: u32 = 0x00003000; // 12 kB

// INTERNAL_USER_RAM_BASE = 0xBF000000 + BMXDUDBA
pub const DRM_SIZE: u32 = 0x00008000; // 32 KB

// virtual addr
pub const BOOT_VIRT_ADDR: u32 = 0xBFC00000;
// pub const KUSEG_BASE: u32 = 0x00000000;
// pub const KUSEG_SIZE: u32 = 0x80000000;
pub const KSEG0_BASE: u32 = 0x80000000;
pub const KSEG0_SIZE: u32 = 0x20000000;
pub const KSEG1_BASE: u32 = 0xA0000000;
pub const KSEG1_SIZE: u32 = 0x20000000;
// pub const KSEG2_BASE: u32 = 0xC0000000;
// pub const KSEG2_SIZE: u32 = 0x40000000;

macro_rules! imm {
    ($a:expr) => {{
        $a & 0x0000ffff
    }};
}

macro_rules! extend {
    ($a:expr, $typ:ty) => {{
        (($a & 0x0000ffff) as $typ) as u32
    }};
}

pub struct Processor {
    pub pc: u32,
    regs: [u32; 32],
    pub drm: Vec<u8>,
    pub bfm: Vec<u8>,
    hi: u32,
    lo: u32,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            pc: BOOT_VIRT_ADDR,
            regs: [0; 32],
            drm: vec![0; DRM_SIZE as usize],
            bfm: vec![0; BFM_SIZE as usize],
            hi: 0u32,
            lo: 0u32,
        }
    }

    pub fn get_inst(&mut self) -> Result<u32, ()> {
        let memtype_addr = self.addr_loc(self.pc);
        match self.load32(memtype_addr.0, memtype_addr.1) {
            Ok(inst) => Ok(inst),
            Err(_e) => Err(_e),
        }
    }

    pub fn write_to_bfm(&mut self, bytes: &[u8]) {
        self.bfm.splice(..bytes.len(), bytes.iter().cloned());
    }


    pub fn handle_instruction(&mut self, inst: u32) -> bool {
        let opcode = ((inst & 0xFC000000) >> 26) as u8;
        let rs = ((inst & 0x3E00000) >> 21) as usize;
        let rt = ((inst & 0x1F0000) >> 16) as usize;
        match opcode {
            0x0 => self.handle_r_instruction(inst),
            0x1 => self.handle_i_branch_instruction(inst),
            0x2 => {
                // J
                self.pc = (self.pc & 0xF0000000) | (inst & 0x3FFFFFF) << 2;
                true
            }
            0x3 => {
                // JAL
                self.regs[31] = self.pc + 4;
                self.pc = (self.pc & 0xF0000000) | (inst & 0x3FFFFFF) << 2;
                true
            }
            0x04 => {
                // BEQ

                let offset = extend!(inst, i16);
                if self.regs[rs] == self.regs[rt] {
                    self.pc = self.pc + (offset << 2);
                }
                true
            }

            0x05 => {
                // BNE
                let offset = extend!(inst, i16);
                if self.regs[rs] != self.regs[rt] {
                    self.pc = self.pc + (offset << 2);
                }
                true
            }
            0x06 => {
                // BLEZ
                let offset = extend!(inst, i16);
                if (self.regs[rs] as i32) <= 0 {
                    self.pc = self.pc + (offset << 2);
                }
                true
            }
            0x07 => {
                // BGTZ
                let offset = extend!(inst, i16);
                if (self.regs[rs] as i32) > 0 {
                    self.pc = self.pc + (offset << 2);
                }
                true
            }
            0x09 => {
                // ADDIU

                let imm = extend!(inst, i16);
                self.regs[rt] = self.regs[rs] + (imm);
                false
            }

            0x0a => {
                // SLTI
                if (self.regs[rs] as i32) < imm!(inst) as i32 {
                    self.regs[rt] = 1u32;
                } else {
                    self.regs[rt] = 0u32;
                }
                false
            }
            0x0b => {
                // SLTIU

                if self.regs[rs] < imm!(inst) {
                    self.regs[rt] = 1u32;
                } else {
                    self.regs[rt] = 0u32;
                }
                false
            }
            0x0c => {
                // ANDI

                self.regs[rt] = self.regs[rs] & imm!(inst);
                false
            }
            0x0d => {
                // ORI

                self.regs[rt] = self.regs[rs] | imm!(inst);
                false
            }
            0x0e => {
                // XORI

                self.regs[rt] = self.regs[rs] ^ imm!(inst);
                false
            }
            0x0f => {
                // LUI

                self.regs[rt] = imm!(inst) << 16;
                false
            }

            0x20 => {
                // LB

                let imm = extend!(inst, i16);
                let memtype_addr = self.addr_loc(self.regs[rs] + (imm));
                let index = memtype_addr.1 as usize;
                match memtype_addr.0 {
                    MemType::BFM => {
                        self.regs[rt] = ((self.bfm[index] & 0x000000ff) as i8) as u32;
                    }
                    MemType::DRM => {
                        self.regs[rt] = ((self.drm[index] & 0x000000ff) as i8) as u32;
                    }
                }

                false
            }

            0x21 => {
                // LH

                let imm = extend!(inst, i16);
                let memtype_addr = self.addr_loc(self.regs[rs] + (imm));
                let index = memtype_addr.1 as usize;
                match memtype_addr.0 {
                    MemType::BFM => {
                        self.regs[rt] = extend!(
                            ((self.bfm[index] as u32) | ((self.bfm[index + 1] as u32) << 8)),
                            i16
                        );
                    }
                    MemType::DRM => {
                        self.regs[rt] = extend!(
                            ((self.drm[index] as u32) | ((self.drm[index + 1] as u32) << 8)),
                            i16
                        );
                    }
                }

                false
            }

            0x23 => {
                //LW

                let imm = extend!(inst, i16);
                let memtype_addr = self.addr_loc(self.regs[rs] + (imm));
                self.regs[rt] = self.load32(memtype_addr.0, self.regs[rs] + (imm)).unwrap();
                false
            }

            0x24 => {
                //LBU

                let imm = extend!(inst, i16);
                let memtype_addr = self.addr_loc(self.regs[rs] + (imm));
                let index = memtype_addr.1 as usize;
                match memtype_addr.0 {
                    MemType::BFM => {
                        self.regs[rt] = self.bfm[index] as u32;
                    }
                    MemType::DRM => {
                        self.regs[rt] = self.drm[index] as u32;
                    }
                }

                false
            }

            0x25 => {
                //LHU

                let imm = extend!(inst, i16);
                let memtype_addr = self.addr_loc(self.regs[rs] + (imm));
                let index = memtype_addr.1 as usize;
                match memtype_addr.0 {
                    MemType::BFM => {
                        self.regs[rt] =
                            (self.bfm[index] as u32) | ((self.bfm[index + 1] as u32) << 8);
                    }
                    MemType::DRM => {
                        self.regs[rt] =
                            (self.bfm[index] as u32) | ((self.bfm[index + 1] as u32) << 8);
                    }
                }

                false
            }

            0x28 => {
                // SB
                let imm = extend!(inst, i16);
                let memtype_addr = self.addr_loc(self.regs[rs] + (imm));
                self.store32(
                    memtype_addr.0,
                    self.regs[rs] + (imm),
                    self.regs[rt] & 0x000000FF,
                );
                false
            }

            0x29 => {
                // SH
                let imm = extend!(inst, i16);
                let memtype_addr = self.addr_loc(self.regs[rs] + (imm));
                self.store32(
                    memtype_addr.0,
                    self.regs[rs] + (imm),
                    self.regs[rt] & 0x0000FFFF,
                );
                false
            }

            0x2b => {
                // SW

                let imm = extend!(inst, i16);
                let memtype_addr = self.addr_loc(self.regs[rs] + (imm));
                self.store32(memtype_addr.0, self.regs[rs] + (imm), self.regs[rt]);
                false
            }
            _ => panic!("Unknown instruction!"),
        }
    }

    fn handle_r_instruction(&mut self, inst: u32) -> bool {
        let rs = ((inst & 0x3E00000) >> 21) as usize;
        let rt = ((inst & 0x1F0000) >> 16) as usize;
        let rd = ((inst & 0xF800) >> 11) as usize;
        let shamt = (inst & 0x7C0) >> 6;
        let funct = inst & 0x3F;
        match funct {
            0x0 => {
                // SLL
                self.regs[rd] = self.regs[rt] << shamt;
                false
            }
            0x2 => {
                // SRL
                self.regs[rd] = self.regs[rt] >> shamt;
                false
            }
            0x3 => {
                // SRA
                self.regs[rd] = ((self.regs[rt] as i32) >> shamt) as u32;
                false
            }
            0x4 => {
                // SLLV

                let shift = self.regs[rs] & 0x1F;
                self.regs[rd] = self.regs[rt] << shift;
                false
            }

            0x6 => {
                // SRLV

                let shift = self.regs[rs] & 0x1F;
                self.regs[rd] = self.regs[rt] >> shift;
                false
            }
            0x7 => {
                // SRAV

                let shift = self.regs[rs] & 0x1F;
                self.regs[rd] = ((self.regs[rt] as i32) >> shift) as u32;
                true
            }
            0x8 => {
                //JR
                self.pc = self.regs[rs];
                true
            }
            0x9 => {
                // JALR
                self.regs[31] = self.pc + 4;
                self.pc = self.regs[rs];
                true
            }
            0x10 => {
                // MFHI

                self.regs[rd] = self.hi;
                false
            }
            0x11 => {
                //MTHI

                self.hi = self.regs[rs];
                false
            }
            0x12 => {
                //MFLO

                self.regs[rd] = self.lo;
                false
            }

            0x13 => {
                //MTLO

                self.lo = self.regs[rs];
                false
            }

            0x18 => {
                // MULT

                let product =
                    (((self.regs[rs] as i32) as i64) * ((self.regs[rt] as i32) as i64)) as u64;
                self.hi = (product >> 32) as u32;
                self.lo = product as u32;
                false
            }
            0x19 => {
                // MULTU
                let product = ((self.regs[rs] as u64) * (self.regs[rt] as u64)) as u64;
                self.hi = (product >> 32) as u32;
                self.lo = product as u32;
                false
            }

            0x20 => {
                //ADD

                self.regs[rd] = ((self.regs[rs] as i32) + (self.regs[rt] as i32)) as u32;
                false
            }
            0x21 => {
                //ADDU

                self.regs[rd] = self.regs[rs] + self.regs[rt];
                false
            }

            0x22 => {
                //SUB

                self.regs[rd] = ((self.regs[rs] as i32) - (self.regs[rt] as i32)) as u32;
                false
            }
            0x23 => {
                //SUBU

                self.regs[rd] = self.regs[rs] - self.regs[rt];
                false
            }
            0x24 => {
                //AND

                self.regs[rd] = self.regs[rs] & self.regs[rt];
                false
            }
            0x25 => {
                // OR

                self.regs[rd] = self.regs[rs] | self.regs[rt];
                false
            }
            0x26 => {
                // XOR

                self.regs[rd] = self.regs[rs] ^ self.regs[rt];
                false
            }
            0x27 => {
                // NOR

                self.regs[rd] = !(self.regs[rs] | self.regs[rt]);
                false
            }
            0x1A => {
                // DIV

                let product =
                    (((self.regs[rs] as i32) as i64) / ((self.regs[rt] as i32) as i64)) as u64;
                self.hi = (product >> 32) as u32;
                self.lo = product as u32;
                false
            }
            0x1B => {
                // DIVU
                let product = ((self.regs[rs] as u64) / (self.regs[rt] as u64)) as u64;
                self.hi = (product >> 32) as u32;
                self.lo = product as u32;
                false
            }

            0x2A => {
                // SLT
                if (self.regs[rs] as i32) < (self.regs[rt] as i32) {
                    self.regs[rd] = 1;
                } else {
                    self.regs[rd] = 0;
                }
                false
            }

            0x2B => {
                // SLTU

                if self.regs[rs] < self.regs[rt] {
                    self.regs[rd] = 1;
                } else {
                    self.regs[rd] = 0;
                }

                false
            }

            0xC => {
                dbg!(format!("not implemented yet: opcode"));
                false

            }
            _ => panic!("Unknown R Type instruction"),
        }
    }

    fn handle_i_branch_instruction(&mut self, inst: u32) -> bool {
        let rs = ((inst & 0x3E00000) >> 21) as usize;
        let rt = ((inst & 0x1F0000) >> 16) as usize;
        match rt {
            0x0 => {
                // BLTZ
                let offset = extend!(inst, i16);
                if (self.regs[rs] as i32) < 0 {
                    self.pc = self.pc + (offset << 2);
                }

                true
            }
            0x1 => {
                // BGEZ
                let offset = extend!(inst, i16);
                if (self.regs[rs] as i32) >= 0 {
                    self.pc = self.pc + (offset << 2);
                }

                true
            }
            0x10 => {
                //BLTZAL
                let offset = extend!(inst, i16);
                if (self.regs[rs] as i32) < 0 {
                    self.regs[31] = self.pc + 4;
                    self.pc = self.pc + (offset << 2);
                }

                true
            }
            0x11 => {
                //BGEZAL

                let offset = extend!(inst, i16);
                if (self.regs[rs] as i32) >= 0 {
                    self.regs[31] = self.pc + 4;
                    self.pc = self.pc + (offset << 2);
                }

                true
            }
            _ => panic!("Uknown branch instruction for REGIMM => {:#08b}", rt),
        }
    }
}
