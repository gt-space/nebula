// use libc::{c_int, c_void, off_t, size_t};
// use std::{
//   ffi::CString,
//   sync::{Arc, Mutex},
// };

// const GPIO_BASE_REGISTERS: [off_t; 4] =
//   [0x44E0_7000, 0x4804_C000, 0x481A_C000, 0x481A_E000];
// const GPIO_REGISTER_SIZE: size_t = 0xFFF;

// const GPIO_OE_REGISTER: isize = 0x134;
// const GPIO_DATAOUT_REGISTER: isize = 0x13C;
// const GPIO_DATAIN_REGISTER: isize = 0x138;

// #[derive(Debug, PartialEq)]
// pub enum PinValue {
//   Low = 0,
//   High = 1,
// }

// #[derive(Debug)]
// pub enum PinMode {
//   Output,
//   Input,
// }

// pub struct Gpio<'a> {
//   fd: c_int,
//   base: *mut c_void,
//   direction: &'a AtomicU32,
//   dataout: &'a AtomicU32,
//   datain: *const u32,
// }

// pub struct Pin<'a> {
//   gpio: &'a Gpio,
//   index: u8,
// }

// // 

// impl Drop for Gpio {
//   fn drop(&mut self) {
//     unsafe {
//       libc::munmap(*self.base.lock().unwrap(), GPIO_REGISTER_SIZE);
//       libc::close(self.fd);
//     };
//   }
// }

// impl Gpio {
//   pub fn open_controller(controller_index: usize) -> Gpio {
//     let path = CString::new("/dev/mem").unwrap();
//     let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };

//     if fd < 0 {
//       panic!("Cannot open memory device");
//     }

//     // /dev/mem accesses physical memory. mmap puts files in address space
//     // but dev/mem is unqiue because contents are actual hw locations

//     /*
//     mmap creates a block of virtual memory that is mapped to the hw registers.
//     Here we cannot interact with the actual register locations directly
//     because there is an OS in the way so we use the virtual memory locations.
//     When we pass the virtual memory location values, the OS will know how
//     to map those operations to the actual register locations because of the
//     relationship that was defined through the mmap call. The return value of
//     mmap is a pointer to the start address of the virtual memory block.
//      */
//     let base = unsafe {
//       libc::mmap(
//         std::ptr::null_mut(),
//         GPIO_REGISTER_SIZE,
//         libc::PROT_READ | libc::PROT_WRITE,
//         libc::MAP_SHARED,
//         fd,
//         GPIO_BASE_REGISTERS[controller_indexindex],
//       )
//     };

//     if base.is_null() {
//       panic!("Cannot map GPIO");
//     }

//     // convert to a u32 reference and make lifetime of self lifetime

//     // if base != GPIO_BASE_REGISTERS[controller_index] as *mut c_void {
//     //   panic!("Invalid start address for GPIO DMA operations");
//     // }
//     // } else if base != GPIO_BASE_REGISTERS[index] as *mut c_void {
//     //     panic!("Cannot acquire GPIO at {index}. Did you call Gpio::open
//     // twice?"); }

//     /*
//     All of the following registers are 32 bits wide, hence the u32 cast.
//     These values are still pointers to virtual memory addresses
//      */
//     let oe = unsafe { base.offset(GPIO_OE_REGISTER) as *mut u32 };

//     let dataout = unsafe { base.offset(GPIO_DATAOUT_REGISTER) as *mut u32 };

//     // changed cast from *mut u32 to *const u32 because it is only being read
//     let datain = unsafe { base.offset(GPIO_DATAIN_REGISTER) as *const u32 };

//     Gpio {
//       fd,
//       base,
//       oe,
//       dataout,
//       datain,
//     }
//   }

//   pub fn get_pin(&self, pin_index: usize) -> Pin {
//     Pin {
//       gpio: &self,
//       pin_index,
//     }
//   }
// }

// impl Pin {
//   pub fn mode(&mut self, mode: PinMode) {
//     let mut bits = unsafe { std::ptr::read_volatile(*(self.gpio.oe)) };

//     bits = match mode {
//       PinMode::Output => bits & !(1 << self.index),
//       PinMode::Input => bits | (1 << self.index),
//     };

//     unsafe { std::ptr::write_volatile(*(self.gpio.oe), bits) };
//   }

//   pub fn digital_write(&mut self, value: PinValue) {
//     let mut bits = unsafe { std::ptr::read_volatile(*(self.gpio.dataout)) };

//     bits = match value {
//       PinValue::Low => bits & !(1 << self.index),
//       PinValue::High => bits | (1 << self.index),
//     };

//     unsafe {std::ptr::write_volatile(*(self.gpio.dataout), bits) };
//   }

//   // just cuz it works doesent mean no undefined behavior

//   pub fn digital_read(&self) -> PinValue {
//     let bits = unsafe { std::ptr::read_volatile(*(self.gpio.datain)) };

//     if bits & (1 << self.index) != 0 {
//       PinValue::High
//     } else {
//       PinValue::Low
//     }
//   }
// }



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

//use common::comm::PinValue;
use libc::{c_int, c_void, off_t, size_t};

use std::ffi::CString;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::rc::Rc;

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
  base: AtomicPtr<c_void>,
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

impl<'a> Drop for Gpio<'a> {
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

impl<'a> Gpio<'a> {
  pub fn open(index: usize) -> Rc<Gpio<'a>> {
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
    }; // when doing offset this increments by bytes

    if base.is_null() {
      panic!("Cannot map GPIO");
    }
    // } else if base != GPIO_BASE_REGISTERS[index] as *mut c_void {
    //     panic!("Cannot acquire GPIO at {index}. Did you call Gpio::open
    // twice?"); }

    // Initialize the AtomicPtr with the memory-mapped base address
    let base = AtomicPtr::new(base);

    // Load the base pointer value to calculate register addresses
    let base_loaded: *mut u8 = base.load(Ordering::SeqCst) as *mut u8;

    // let oe = Mutex::new(unsafe { base.offset(GPIO_OE_REGISTER) as *mut u32 });
    let oe_address = unsafe { base_loaded.offset(GPIO_OE_REGISTER) as *mut AtomicU32 };
    let oe = unsafe { &*oe_address };

    // let dataout =
    //   Mutex::new(unsafe { base.offset(GPIO_DATAOUT_REGISTER) as *mut u32 });
    
    let dataout_address = unsafe { base_loaded.offset(GPIO_DATAOUT_REGISTER) as *mut AtomicU32 };
    let dataout = unsafe{ &*dataout_address} ;

    // let datain = unsafe { base.offset(GPIO_DATAIN_REGISTER) as *mut u32 };
    let datain_address = unsafe { base_loaded.offset(GPIO_DATAIN_REGISTER) as *const AtomicU32 };
    let datain = unsafe { &*datain_address };

    Rc::new(Gpio {
      fd,
      base,
      oe,
      dataout,
      datain,
    })
  }

  pub fn get_pin(self: &Rc<Self>, index: usize) -> Pin<'a> {
    Pin {
      gpio: self.clone(),
      index,
    }
  }
}

impl<'a> Pin<'a> {
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
      PinMode::Input => {
        self.gpio.oe.fetch_or(bits, Ordering::SeqCst);
      },

      PinMode::Output => {
        self.gpio.oe.fetch_and(!bits, Ordering::SeqCst);
      }
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