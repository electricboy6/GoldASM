use std::io::{Read, Write};
use std::time::Duration;
use serialport::{SerialPortType};

fn parse(data: &[u8]) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    result = data.to_vec();

    //todo!()
    result
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
            println!("DEBUG: Found port: {port:#?}");
            target_port = Some(port);
            break;
        }
    }
    if let Some(port_info) = target_port {
        println!("INFO: Opening port...");
        let mut port = serialport::new(port_info.port_name, 1_000_000)
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One)
            .parity(serialport::Parity::None)
            .timeout(Duration::from_millis(2000))
            .flow_control(serialport::FlowControl::Hardware)
            .open()
            .expect("Failed to open serial port!");
        println!("INFO: Sending data...");
        port.write_all(&parse(data)).expect("Couldn't send the data!");

        println!("INFO: Receiving data...");
        let mut received_data = vec!(0; 1024);
        let bytes_received = port.read(&mut received_data)
            .expect("Couldn't receive the data!");
        let chars_received = received_data.iter().map(
            |byte| *byte as char
        ).collect::<String>();
        println!("INFO: Received {bytes_received} bytes: {received_data:?} chars: {chars_received}");

        println!("Done!");
    } else {
        panic!("No Alchitry FPGA found!")
    }
}
