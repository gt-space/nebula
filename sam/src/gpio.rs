// We are using a memory mapped implementation to increase gpio switching
// frequency
//
// https://kilobaser.com/beaglebone-black-gpios/
// The AM335x has four built-in GPIO controllers, named gpio0[], gpio1[],
// gpio2[] and gpio3[]. For each controller, there is one page of memory which
// controls each gpio controller. Each controller is responsible for 32 GPIOs.
// Each 32bit word has a specific function. Like pin configuration, controlling
// or setting a specific pin-state. Each bit in each of these words controls a
// GPIO pin. Choose function by choosing the word, choose GPIO by choosing the
// bit.
//
// https://kilobaser.com/wp-content/uploads/2021/02/BBB_SRM.pdf
// Table 12 and 13 were used to determine the P[8/9]_pin_number on expansion
// header -> gpio controller value in chip

use common::comm::PinValue;
use libc::{c_int, c_void, off_t, size_t};
use std::{
  ffi::CString,
  sync::{atomic::{AtomicU32, Ordering}, Arc, Mutex},
};

use vol

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

#[derive(Debug)]
pub enum BitOrder {
  LSBFirst,
  MSBFirst,
}

// pub struct Gpio {
//   fd: c_int,
//   base: Mutex<*mut c_void>,
//   oe: Mutex<*mut u32>,
//   dataout: Mutex<*mut u32>,
//   datain: *const u32,
// }

pub struct Gpio<'a> {
  fd: c_int,
  base: AtomicPtr<u8>,
  oe: &'a AtomicU32,
  dataout: &'a AtomicU32,
  datain: &'a AtomicU32
}

// below impl's not needed because we are moving to single threaded application
// unsafe impl Send for Gpio {}
// unsafe impl Sync for Gpio {}

pub struct Pin<'a> {
  // gpio: Arc<Gpio>,
  gpio: Rc<Gpio<'a>>,
  index: usize,
}

// impl Drop for Gpio {
//   fn drop(&mut self) {
//     unsafe {
//       libc::munmap(*self.base.lock().unwrap(), GPIO_REGISTER_SIZE);
//       libc::close(self.fd);
//     };
//   }
// }

impl Drop for Gpio {
  fn drop(&mut self) {
    unsafe {
      // Load the base pointer from AtomicPtr with strictest ordering
      let base_ptr = self.base.load(Ordering::SeqCst);

      // Unmap the memory and close the file descriptor
      libc::munmap(base_ptr, GPIO_REGISTER_SIZE);
      libc::close(self.fd);

    }
  }
}

impl Gpio {
  pub fn open(index: usize) -> Rc<Gpio> {
    let path = CString::new("/dev/mem").unwrap();
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };

    if fd < 0 {
      panic!("Cannot open memory device");
    }

    let base = unsafe {
      libc::mmap(
        std::ptr::null_mut(),
        GPIO_REGISTER_SIZE,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
        fd,
        GPIO_BASE_REGISTERS[index],
      )
    } as *mut u8; // when doing offset this increments by bytes

    if base.is_null() {
      panic!("Cannot map GPIO");
    }
    // } else if base != GPIO_BASE_REGISTERS[index] as *mut c_void {
    //     panic!("Cannot acquire GPIO at {index}. Did you call Gpio::open
    // twice?"); }

    // Initialize the AtomicPtr with the memory-mapped base address
    let base = AtomicPtr::new(base_ptr);

    // Load the base pointer value to calculate register addresses
    let base_loaded = base.load(Ordering::SeqCst);

    // let oe = Mutex::new(unsafe { base.offset(GPIO_OE_REGISTER) as *mut u32 });
    let oe_address = unsafe { base_loaded.ofset(GPIO_OE_REGISTER) as *mut AtomicU32 };
    let oe = unsafe { &*oe_address };

    // let dataout =
    //   Mutex::new(unsafe { base.offset(GPIO_DATAOUT_REGISTER) as *mut u32 });
    
    let dataout_address = unsafe { base_loaded.ofset(GPIO_DATAOUT_REGISTER) as *mut AtomicU32 };
    let dataout = unsafe{ &*dataout_address} ;

    // let datain = unsafe { base.offset(GPIO_DATAIN_REGISTER) as *mut u32 };
    let datain_address = unsafe { base_loaded.offset(GPIO_DATAIN_REGISTER) as *const AtomicU32 };
    let datain = unsafe { &*datain_address };

    let base = Mutex::new(base);

    Rc::new(Gpio {
      fd,
      base,
      oe,
      dataout,
      datain,
    })
  }

  pub fn get_pin(self: &Arc<Self>, index: usize) -> Pin {
    Pin {
      gpio: self.clone(),
      index,
    }
  }
}

impl Pin {
  // pub fn mode(&self, mode: PinMode) {
  //   let oe = self.gpio.oe.lock().unwrap();
  //   let mut bits = unsafe { std::ptr::read_volatile(*oe) };

  //   bits = match mode {
  //     PinMode::Output => bits & !(1 << self.index),
  //     PinMode::Input => bits | (1 << self.index),
  //   };

  //   unsafe { std::ptr::write_volatile(*oe, bits) };
  // }

  pub fn mode(&self, mode: PinMode) {
    let bits = 1 << self.index;
    match mode {
      PinMode::Input => self.gpio.oe.fetch_or(bits, Ordering::SeqCst),
      PinMode::Output => self.gpio.oe.fetch_and(!bit, Ordering::SeqCst)
    }
  }

  // pub fn digital_write(&self, value: PinValue) {
  //   let dataout = self.gpio.dataout.lock().unwrap();
  //   let mut bits = unsafe { std::ptr::read_volatile(*dataout) };

  //   bits = match value {
  //     PinValue::Low => bits & !(1 << self.index),
  //     PinValue::High => bits | (1 << self.index),
  //   };

  //   unsafe { std::ptr::write_volatile(*dataout, bits) };
  // }
  pub fn digital_write(&self, value: PinValue) {
    let bits = 1 << self.index;
    match value {
      PinValue::Low => {
        self.gpio.dataout.fetch_and(!bits, Ordering::SeqCst);
      },

      PinValue::High => {
        self.gpio.dataout.fetch_or(bits, Ordering::SeqCst);
      }
    }
  }

  // pub fn digital_read(&self) -> PinValue {
  //   let datain = self.gpio.datain;
  //   let bits = unsafe { std::ptr::read_volatile(datain) };

  //   if bits & (1 << self.index) != 0 {
  //     PinValue::High
  //   } else {
  //     PinValue::Low
  //   }
  // }

  pub fn digital_read(&self) -> PinValue {
    let datain = self.gpio.datain.load(Ordering::SeqCst);

    if datain & (1 << self.index) != 0 {
      PinValue::High
    } else {
      PinValue::Low
    }
  }
}
