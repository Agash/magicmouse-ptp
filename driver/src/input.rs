use crate::descriptors::MagicMouseInputReport;
use wdk::println;
use wdk_sys::WDFDEVICE;

pub unsafe fn process_input_report(
    _device: WDFDEVICE,
    raw: &MagicMouseInputReport,
    len: usize,
) {
    if len > 8 {
        println!("MM_SNIFFER: Report 0x10. Buttons: {}", raw.buttons);
    }
}