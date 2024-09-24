use libc::{c_int, c_void, off_t, size_t};
use std::{
  ffi::CString,
  sync::{Arc, Mutex},
};

const GPIO_BASE_REGISTERS: [off_t; 4] =
  [0x44E0_7000, 0x4804_C000, 0x481A_C000, 0x481A_E000];
const GPIO_REGISTER_SIZE: size_t = 0xFFF;

const GPIO_OE_REGISTER: isize = 0x134;
const GPIO_DATAOUT_REGISTER: isize = 0x13C;
const GPIO_DATAIN_REGISTER: isize = 0x138;

#[derive(Debug, PartialEq)]
pub enum PinValue {
  Low = 0,
  High = 1,
}

#[derive(Debug)]
pub enum PinMode {
  Output,
  Input,
}

pub struct Gpio {
  fd: c_int,
  base: *mut c_void,
  oe: *mut u32,
  dataout: *mut u32,
  datain: *const u32,
}

pub struct Pin {
  gpio: &Gpio,
  index: usize,
}

impl Drop for Gpio {
  fn drop(&mut self) {
    unsafe {
      libc::munmap(*self.base.lock().unwrap(), GPIO_REGISTER_SIZE);
      libc::close(self.fd);
    };
  }
}

impl Gpio {
  pub fn open_controller(controller_index: usize) -> &Gpio {
    let path = CString::new("/dev/mem").unwrap();
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };

    if fd < 0 {
      panic!("Cannot open memory device");
    }

    /*
    mmap creates a block of virtual memory that is mapped to the hw registers.
    Here we cannot interact with the actual register locations directly
    because there is an OS in the way so we use the virtual memory locations.
    When we pass the virtual memory location values, the OS will know how
    to map those operations to the actual register locations because of the
    relationship that was defined through the mmap call. The return value of
    mmap is a pointer to the start address of the virtual memory block.
     */
    let base = unsafe {
      libc::mmap(
        std::ptr::null_mut(),
        GPIO_REGISTER_SIZE,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
        fd,
        GPIO_BASE_REGISTERS[controller_indexindex],
      )
    } as *const u8;

    if base.is_null() {
      panic!("Cannot map GPIO");
    }

    // if base != GPIO_BASE_REGISTERS[controller_index] as *mut c_void {
    //   panic!("Invalid start address for GPIO DMA operations");
    // }
    // } else if base != GPIO_BASE_REGISTERS[index] as *mut c_void {
    //     panic!("Cannot acquire GPIO at {index}. Did you call Gpio::open
    // twice?"); }

    /*
    All of the following registers are 32 bits wide, hence the u32 cast.
    These values are still pointers to virtual memory addresses
     */
    let oe = unsafe { base.offset(GPIO_OE_REGISTER) as *mut u32 };

    let dataout = unsafe { base.offset(GPIO_DATAOUT_REGISTER) as *mut u32 };

    // changed cast from *mut u32 to *const u32 because it is only being read
    let datain = unsafe { base.offset(GPIO_DATAIN_REGISTER) as *const u32 };

    Gpio {
      fd,
      base,
      oe,
      dataout,
      datain,
    }
  }

  pub fn get_pin(&self, pin_index: usize) -> Pin {
    Pin {
      gpio: &self,
      pin_index,
    }
  }
}

impl Pin {
  pub fn mode(&mut self, mode: PinMode) {
    let mut bits = unsafe { std::ptr::read_volatile(*(self.gpio.oe)) };

    bits = match mode {
      PinMode::Output => bits & !(1 << self.index),
      PinMode::Input => bits | (1 << self.index),
    };

    unsafe { std::ptr::write_volatile(*(self.gpio.oe), bits) };
  }

  pub fn digital_write(&mut self, value: PinValue) {
    let mut bits = unsafe { std::ptr::read_volatile(*(self.gpio.dataout)) };

    bits = match value {
      PinValue::Low => bits & !(1 << self.index),
      PinValue::High => bits | (1 << self.index),
    };

    unsafe {std::ptr::write_volatile(*(self.gpio.dataout), bits) };
  }

  pub fn digital_read(&self) -> PinValue {
    let bits = unsafe { std::ptr::read_volatile(*(self.gpio.datain)) };

    if bits & (1 << self.index) != 0 {
      PinValue::High
    } else {
      PinValue::Low
    }
  }
}