#![no_std]
#![no_main]

use core::arch::asm;
use embedded_graphics::{
    image::Image,
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use opensbi_rt::*;
use tinybmp::Bmp;
use uart_16550::MmioSerialPort;
use virtio_drivers::VirtIOHeader;

struct Framebuffer {
    addr: *mut u8,
}

impl DrawTarget for Framebuffer {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if let Ok((x @ 0..=640, y @ 0..=480)) = coord.try_into() {
                // Calculate the index in the framebuffer.
                let index: u32 = (x + y * 640) * 2;
                let color = color.to_le_bytes();

                unsafe {
                    self.addr.add(index as usize).copy_from(color.as_ptr(), 2);
                }
            }
        }

        Ok(())
    }
}

impl OriginDimensions for Framebuffer {
    fn size(&self) -> Size {
        Size::new(640, 480)
    }
}

#[opensbi_rt::entry]
fn main(hartid: usize, dtb_pa: usize) {
    println!("{:#x}", dtb_pa);
    if let Ok(dt) = unsafe { fdt::Fdt::from_ptr(dtb_pa as *const u8) } {
        // let node = dt
        //     .all_nodes()
        //     .find(|v| v.name.contains("framebuffer"))
        //     .and_then(|v| v.reg())
        //     .and_then(|mut v| v.next())
        //     .unwrap();
        // let mut fb = Framebuffer {
        //     addr: node.starting_address as *mut u8,
        // };
        // fb.clear(Rgb565::BLACK).unwrap();

        // let bmp_data = include_bytes!("../trans_rights.bmp");
        // let bmp = Bmp::<Rgb565>::from_slice(bmp_data).unwrap();
        // Image::new(&bmp, Point::new(0, 0)).draw(&mut fb).unwrap();

        // let style = MonoTextStyle::new(&FONT_10X20, Rgb565::BLACK);
        // Text::new("nyah~<3", Point::new(20, 30), style)
        //     .draw(&mut fb)
        //     .unwrap();

        // loop {}
        let uart = dt.find_node("/soc/uart").unwrap();
        let uart_addr = uart
            .reg()
            .and_then(|mut v| v.next())
            .unwrap()
            .starting_address as usize;
        let mut serial_port = unsafe { MmioSerialPort::new(uart_addr) };
        serial_port.init();
        let s = "hi from rust! nya~";
        s.chars().for_each(|c| serial_port.send(c as u8));
    }
}

fn virtio(node: &fdt::node::FdtNode) {
    if let Some(reg) = node.reg().and_then(|mut v| v.next()) {
        let header = unsafe { &mut *(reg.starting_address as *mut VirtIOHeader) };
        println!("{:?}", header.device_type());
        println!("---");
    }
}
