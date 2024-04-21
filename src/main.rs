#![windows_subsystem = "windows"]

extern crate serialport;
extern crate sysinfo;
extern crate systemstat;
extern crate serde;

use machine_info::Machine;
use serialport::SerialPort;
use std::io::Read;
use std::{fs, thread};
use std::time::Duration;
use sysinfo::Networks;
use systemstat::{Platform, System};

use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, TrayIconEvent,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ConfigToml{
    port: String,
}

fn main() {
    thread::spawn(|| {
        analog_control();
    });
    //handle.join().unwrap();
    tray_icon();
}

// Analog control functions

fn analog_control() {
    let mut sys: System;
    let mut m: Machine;
    let mut networks = Networks::new_with_refreshed_list();

    let mut used_mem_per: f64;
    let mut used_cpu_per: f32;
    let mut used_gpu_per: f32;
    let mut used_net_per: f32;

    let mut memory: systemstat::Memory;
    let mut cpu: systemstat::DelayedMeasurement<systemstat::CPULoad>;
    let mut graphics: Vec<machine_info::GraphicsUsage>;
    let mut upload: u64;
    let mut download: u64;
    let mut sending_data: u8;

    let config_str = fs::read_to_string("config.toml").expect("Fail to open config");
    let config_toml: ConfigToml = toml::from_str(&config_str).expect("Fallo al deserializar");

    let mut serial_port: Box<dyn SerialPort> = serialport::new(config_toml.port, 9600)
        .timeout(Duration::from_millis(10000))
        .open()
        .expect("Failed to open serial port");
    thread::sleep(Duration::from_millis(1000));

    // RESET SIGNAL
    send(0, &mut serial_port);
    thread::sleep(Duration::from_millis(1500));

    loop {
        sys = System::new();
        m = Machine::new();

        cpu = sys.cpu_load_aggregate().unwrap();
        thread::sleep(Duration::from_millis(500));
        let cpu_nums = cpu.done().unwrap();
        used_cpu_per = 1.0 - cpu_nums.idle;
        sending_data = ((used_cpu_per * 230.0) + 25.0) as u8;
        send(sending_data, &mut serial_port);
        println!("Output (cpu): {}", sending_data);
        //thread::sleep(Duration::from_millis(1000));

        memory = sys.memory().unwrap();
        used_mem_per = 1.0 - (memory.free.as_u64() as f64 / memory.total.as_u64() as f64);
        sending_data = ((used_mem_per * 230.0) + 25.0) as u8;
        send(sending_data, &mut serial_port);
        println!("Output (mem): {}", sending_data);
        //thread::sleep(Duration::from_millis(1000));

        graphics = m.graphics_status();
        used_gpu_per = graphics[0].gpu as f32 / 100.0;
        sending_data = ((used_gpu_per * 230.0) + 25.0) as u8;
        send(sending_data, &mut serial_port);
        println!("Output (gpu): {:?}", sending_data);
        //thread::sleep(Duration::from_millis(1000));
        
        download = networks.get("Ethernet").unwrap().received();
        upload = networks.get("Ethernet").unwrap().transmitted();
        // WHY 7.5 NEEDED???????
        used_net_per = (7.5 * (download + upload) as f32) / 1000000000.0;
        sending_data = ((used_net_per * 230.5) + 25.0) as u8;
        send(sending_data, &mut serial_port);
        println!("Output (net): {}", sending_data);
        networks.refresh();
        // thread::sleep(Duration::from_millis(1000));
        
        println!("-------");
        receive(&mut serial_port);
        println!("-------");

        //thread::sleep(Duration::from_millis(1000));
    }
}

fn send(data: u8, serial_port: &mut Box<dyn SerialPort>) {
    serial_port.write(&[data]).expect("Write failed!");
    serial_port.flush().unwrap();
}

fn receive(serial_port: &mut Box<dyn SerialPort>) {
    println!("Receiving...");
    let mut serial_buf: Vec<u8> = vec![0; 4];
    serial_port
        .read(serial_buf.as_mut_slice())
        .expect("Found no data!");
    println!("{:?}", serial_buf);
    println!("Received!");
}

// Tray icon fcuntions

fn tray_icon() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/icon.png");
    let event_loop = EventLoopBuilder::new().build();
    let tray_menu = Menu::new();
    let quit_i = MenuItem::new("Quit", true, None);
    let _ = tray_menu.append_items(&[
        &PredefinedMenuItem::about(
            None,
            Some(AboutMetadata {
                name: Some("EasyAnalogSystemInfo - Gestor de medidores analógicos".to_string()),
                copyright: Some("Copyright JM4000".to_string()),
                ..Default::default()
            }),
        ),
        &PredefinedMenuItem::separator(),
        &quit_i,
    ]);
    let mut tray_icon = None;
    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let tao::event::Event::NewEvents(tao::event::StartCause::Init) = event {
            let icon = load_icon(std::path::Path::new(path));

            // We create the icon once the event loop is actually running
            // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
            tray_icon = Some(
                TrayIconBuilder::new()
                    .with_menu(Box::new(tray_menu.clone()))
                    .with_menu_on_left_click(true)
                    .with_icon(icon)
                    .build()
                    .unwrap(),
            );
        }

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_i.id() {
                tray_icon.take();
                *control_flow = ControlFlow::Exit;
            }
            println!("{:?} clicked!", event.id);
        }

        if let Ok(_event) = tray_channel.try_recv() {
            println!("Tray icon clicked!");
        }
    })
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
