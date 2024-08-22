use spidev::spidevioctl::SpidevTransfer;
use spidev::{SpiModeFlags, Spidev, SpidevOptions};
use std::thread;
use std::fs;
use std::time::Duration;
pub mod gpio;

fn main() {
  const IMU_CS: &str = "11";
  const IMU_NRST: &str = "89";
  const IMU_DR: &str = "81";

  const MAG_CS: &str = "46";
  const MAG_DR: &str = "65";
  const MAG_INT: &str = "61";

  const BAR_CS: &str = "88";

  let mut spi = Spidev::open("/dev/spidev0.0").expect("Failed to initalize SPI");
  let mut imu = SpidevOptions::new();
  let mut mag = SpidevOptions::new();
  let mut bar = SpidevOptions::new();

  gpio::set_output(IMU_NRST);
  gpio::set_low(IMU_NRST);
  thread::sleep(Duration::from_micros(200));
  gpio::set_high(IMU_NRST);

  gpio::set_output(IMU_CS);
  gpio::set_high(IMU_CS);

  gpio::set_output(MAG_CS);
  gpio::set_high(MAG_CS);

  gpio::set_output(BAR_CS);
  gpio::set_high(BAR_CS);

  gpio::set_input(IMU_DR);
  gpio::set_input(MAG_DR);
  gpio::set_input(MAG_INT);

  loop {
    configure_imu(&mut spi, &mut imu);
    gpio::set_low(IMU_CS);
    if read_gpio(IMU_DR) == "1" {
      read_imu(&mut spi);
    }
    gpio::set_high(IMU_CS);

    thread::sleep(Duration::from_micros(100));
  }
}

fn read_imu(spi: &mut Spidev) {
  read_spi2([0x1C, 0x00], spi, String::from("TEMP"));
  read_spi2([0x1E, 0x00], spi, String::from("TIME"));

  read_spi4([0x04, 0x00, 0x06, 0x00], spi, String::from("X_GYRO"));
  read_spi4([0x08, 0x00, 0x0A, 0x00], spi, String::from("Y_GYRO"));
  read_spi4([0x0C, 0x00, 0x0E, 0x00], spi, String::from("Z_GYRO"));

  read_spi4([0x10, 0x00, 0x12, 0x00], spi, String::from("X_ACCL"));
  read_spi4([0x14, 0x00, 0x16, 0x00], spi, String::from("Y_ACCL"));
  read_spi4([0x18, 0x00, 0x1A, 0x00], spi, String::from("Z_ACCL"));

  read_spi4([0x24, 0x00, 0x26, 0x00], spi, String::from("X_DELTANG"));
  read_spi4([0x28, 0x00, 0x2A, 0x00], spi, String::from("Y_DELTANG"));
  read_spi4([0x2C, 0x00, 0x2E, 0x00], spi, String::from("Z_DELTANG"));

  read_spi4([0x30, 0x00, 0x32, 0x00], spi, String::from("X_DELTVEL"));
  read_spi4([0x34, 0x00, 0x36, 0x00], spi, String::from("Y_DELTVEL"));
  read_spi4([0x38, 0x00, 0x3A, 0x00], spi, String::from("Z_DELTVEL"));
}

fn read_spi4(tx_buf: [u8; 4], spi: &Spidev, s: String) {
  let mut rx_buf = [0; 4];
  let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
  let result = spi.transfer(&mut transfer);
  match result {
    Ok(_) => {
      println!("{s}: {:?}", rx_buf);
    }
    Err(err) => println!("{:?}", err),
  }
}

fn read_spi2(tx_buf: [u8; 2], spi: &Spidev, s: String) {
  let mut rx_buf = [0; 2];
  let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
  let result = spi.transfer(&mut transfer);
  match result {
    Ok(_) => {
      println!("{s}: {:?}", rx_buf);
    }
    Err(err) => println!("{:?}", err),
  }
}

fn configure_imu(spi: &mut Spidev, imu: &mut SpidevOptions) {
  imu.bits_per_word(8);
  imu.max_speed_hz(2_000_000);
  imu.lsb_first(false);
  imu.mode(SpiModeFlags::SPI_MODE_0);
  imu.build();
  spi.configure(imu).expect("Failed to configure SPI for the IMU");
}

fn configure_mag(spi: &mut Spidev, mag: &mut SpidevOptions) {
  mag.bits_per_word(8);
  mag.max_speed_hz(10_000_000);
  mag.lsb_first(false);
  mag.mode(SpiModeFlags::SPI_MODE_3);
  mag.build();
  spi.configure(mag).expect("Failed to configure SPI for the Magnetometer");
}

fn configure_bar(spi: &mut Spidev, bar: &mut SpidevOptions) {
  bar.bits_per_word(8);
  bar.max_speed_hz(20_000_000);
  bar.lsb_first(false);
  bar.mode(SpiModeFlags::SPI_MODE_0);
  bar.build();
  spi.configure(bar).expect("Failed to configure SPI for the Barometer");
}

fn read_gpio(pin: &str) -> String {
  let path = format!("/sys/class/gpio/gpio{}/value", pin);
  let value = fs::read_to_string(path).expect("Failed to read GPIO");
  value
}