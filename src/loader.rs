use std::io::Write;
use std::time::Duration;
use serialport::{SerialPortType, UsbPortInfo};

fn parse(data: &[u8]) -> Vec<u8> {
    todo!()
}

pub fn send(data: &[u8]) {
    let ports = serialport::available_ports().expect("No ports found! (did you plug in the fpga?)");
    let mut target_port = None;
    for port in ports {
        if let SerialPortType::UsbPort(info) = port.port_type.clone()
            && let Some(ref product_name) = info.product
            && product_name.contains("Alchitry")
            && product_name.contains("V2")
        {
            target_port = Some(port);
            break;
        }
    }
    if let Some(port_info) = target_port {
        let mut port = serialport::new(port_info.port_name, 1_000_000)
            // todo: check if these options are correct
            .dtr_on_open(true)
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One)
            .parity(serialport::Parity::Odd)
            // up to here
            .timeout(Duration::from_millis(500))
            .open()
            .expect("Failed to open serial port!");
        print!("");
        port.write_all(&parse(data)).expect("Couldn't send the data!");
    } else {
        panic!("No Alchitry FPGA found!")
    }
}
