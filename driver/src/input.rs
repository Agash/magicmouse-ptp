use crate::descriptors::MagicMouseInputReport;
use wdk::println;
use wdk_sys::WDFDEVICE;

pub unsafe fn process_input_report(
    _device: WDFDEVICE,
    raw: &MagicMouseInputReport,
    len: usize,
) {
    if len < core::mem::size_of::<MagicMouseInputReport>() {
        println!("process_input_report: Report too short: {} bytes", len);
        return;
    }
    
    // Copy packed fields to local variables to avoid alignment issues
    let report_id = raw.report_id;
    let buttons = raw.buttons;
    let dx = raw.dx;
    let dy = raw.dy;
    let touch_data_0 = raw.touch_data[0];
    
    println!(
        "MM_REPORT: ID={:#04X} Buttons={:#04X} dx={} dy={} TouchData[0]={:#04X}",
        report_id,
        buttons,
        dx,
        dy,
        touch_data_0
    );
    
    // TODO: Parse touch data and create PTP reports
    // For now, just log that we received the report
}