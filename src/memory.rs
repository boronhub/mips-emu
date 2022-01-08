use super::cpu::*;

pub enum MemType {
    BFM,
    DRM,
}

impl Processor {
    pub fn fmt(&self, addr: u32) -> u32 {
        if KSEG0_BASE <= addr && addr < KSEG0_BASE + KSEG0_SIZE {
            return addr - KSEG0_BASE;
        } else if KSEG1_BASE <= addr && addr < KSEG1_BASE + KSEG1_SIZE {
            return addr - KSEG1_BASE;
        } else {
            dbg!("reserved");
        }
        return 0;
    }

    pub fn addr_loc(&self, addr: u32) -> (MemType, u32) {
        let phys_addr = self.fmt(addr);
        if BFM_BASE <= phys_addr && phys_addr < BFM_BASE + BFM_SIZE {
            return (MemType::BFM, phys_addr - BFM_BASE);
        } else {
            return (MemType::DRM, phys_addr);
        }
    }

    pub fn load32(&self, mem_type: MemType, addr: u32) -> Result<u32, ()> {
        let offset = addr as usize;
        match mem_type {
            MemType::BFM => {
                return Ok((self.bfm[offset] as u32)
                    | ((self.bfm[offset + 1] as u32) << 8)
                    | ((self.bfm[offset + 2] as u32) << 16)
                    | ((self.bfm[offset + 3] as u32) << 24));
            }
            MemType::DRM => {
                return Ok((self.drm[offset] as u32)
                    | ((self.drm[offset + 1] as u32) << 8)
                    | ((self.drm[offset + 2] as u32) << 16)
                    | ((self.drm[offset + 3] as u32) << 24));
            }
        }
    }

    pub fn store32(&mut self, mem_type: MemType, addr: u32, value: u32) {
        let offset = addr as usize;
        match mem_type {
            MemType::BFM => {
                self.bfm[offset] = (value & 0xff) as u8;
                self.bfm[offset + 1] = ((value >> 8) & 0xff) as u8;
                self.bfm[offset + 2] = ((value >> 16) & 0xff) as u8;
                self.bfm[offset + 3] = ((value >> 24) & 0xff) as u8;
            }
            MemType::DRM => {
                self.drm[offset] = (value & 0xff) as u8;
                self.drm[offset + 1] = ((value >> 8) & 0xff) as u8;
                self.drm[offset + 2] = ((value >> 16) & 0xff) as u8;
                self.drm[offset + 3] = ((value >> 24) & 0xff) as u8;
            }
        }
    }
}
