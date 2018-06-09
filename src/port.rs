use bit_field::BitField;
use volatile::Volatile;

pub enum PortName {
    C,
}

#[repr(C, packed)]
pub struct Port {
    pcr: [Volatile<u32>; 32],
    gpclr: Volatile<u32>,
    gpchr: Volatile<u32>,
    reserved_0: [u8; 24],
    isfr: Volatile<u32>,
}

pub struct Pin {
    port: *mut Port,
    pin: usize,
}

#[repr(C, packed)]
struct GpioBitband {
    pdor: [Volatile<u32>; 32],
    psor: [Volatile<u32>; 32],
    pcor: [Volatile<u32>; 32],
    ptor: [Volatile<u32>; 32],
    pdir: [Volatile<u32>; 32],
    pddr: [Volatile<u32>; 32],
}

pub struct GpioPin {
    bitband: *mut GpioBitband,
    pin: usize,
}

impl Port {
    pub unsafe fn new(name: PortName) -> &'static mut Port {
        &mut *match name {
            PortName::C => 0x4004_b000 as *mut Port,
            _ => panic!("Invalid port name"),
        }
    }

    pub unsafe fn pin(&mut self, p: usize) -> Pin {
        Pin { port: self, pin: p }
    }

    pub unsafe fn set_pin_mode(&mut self, p: usize, mode: u32) {
        self.pcr[p].update(|pcr| {
            pcr.set_bits(8..11, mode);
        });
    }

    pub fn name(&self) -> PortName {
        let addr = (self as *const Port) as u32;
        match addr {
            0x4004_b000 => PortName::C,
            _ => unreachable!(),
        }
    }
}

pub struct Tx(u8);
pub struct Rx(u8);

impl Tx {
    pub fn uart(&self) -> u8 {
        self.0
    }
}

impl Rx {
    pub fn uart(&self) -> u8 {
        self.0
    }
}

impl Pin {
    pub fn make_gpio(self) -> GpioPin {
        unsafe {
            let port = &mut *self.port;
            port.set_pin_mode(self.pin, 1);
            GpioPin::new(port.name(), self.pin)
        }
    }

    pub fn make_rx(self) -> Rx {
        unsafe {
            let port = &mut *self.port;
            match (port.name(), self.pin) {
                (PortName::C, 16) => {
                    port.set_pin_mode(self.pin, 3);
                    Rx(0)
                }
                _ => panic!("Invalid serial Rx pin"),
            }
        }
    }

    pub fn make_tx(self) -> Tx {
        unsafe {
            let port = &mut *self.port;
            match (port.name(), self.pin) {
                (PortName::C, 17) => {
                    port.set_pin_mode(self.pin, 3);
                    Tx(0)
                }
                _ => panic!("Invalid serial Tx pin"),
            }
        }
    }
}

impl GpioPin {
    pub unsafe fn new(port: PortName, pin: usize) -> GpioPin {
        let bitband = match port {
            PortName::C => 0x43fe_1000 as *mut GpioBitband,
            _ => panic!("Invalid port name"),
        };

        GpioPin { bitband, pin }
    }

    pub fn output(&mut self) {
        unsafe {
            (*self.bitband).pddr[self.pin].write(1);
        }
    }

    pub fn high(&mut self) {
        unsafe {
            (*self.bitband).psor[self.pin].write(1);
        }
    }
}
