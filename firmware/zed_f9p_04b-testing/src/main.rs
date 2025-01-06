use zed_f9p_04b::GPS;
use std::thread::sleep;
use std::time::Duration;

fn main() {
  let bus = "/dev/spidev0.0"; // Adjust this
  let cs_pin = 5; // Adjust this

  println!("Initializing GPS...");
  match GPS::new(bus, cs_pin) {
    Ok(mut gps) => {
      println!("GPS initialized successfully!");

      println!("Getting device version information...");
      if let Err(e) = gps.mon_ver() {
        eprintln!("Failed to get MON-VER message: {}", e);
        return;
      }

      println!("Polling PVT data...");
      loop {
        match gps.poll_pvt() {
          Ok(Some(pvt)) => {
            println!("PVT data received:");
            if let Some(pos) = pvt.position {
              println!(
                "Position: Latitude {:.5}, Longitude {:.5}, Altitude {:.2}m",
                pos.lat, pos.lon, pos.alt
              );
            }
            if let Some(vel) = pvt.velocity {
              println!(
                "Velocity: Speed {:.2} m/s, Heading {:.2}°",
                vel.speed, vel.heading
              );
            }
            if let Some(time) = pvt.time {
              println!("Time: {}", time);
            }
          }
          Ok(None) => {
            println!("No PVT data received.");
          }
          Err(e) => {
            eprintln!("Error polling PVT data: {}", e);
          }
        }
        sleep(Duration::from_millis(100));
      }
    }
    Err(e) => {
      eprintln!("Error initializing GPS: {}", e);
    }
  }
}
