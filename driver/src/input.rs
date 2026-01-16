use crate::device::get_device_context;
use crate::descriptors::{MagicMouseInputReport, MouseInputReport, PTPContact, PTPInputReport, PTPFeatureReport};
use core::ffi::c_void;
use wdk_sys::{
    call_unsafe_wdf_function_binding,
    STATUS_BUFFER_TOO_SMALL, STATUS_SUCCESS, WDFDEVICE, WDFQUEUE, WDFREQUEST,
};

const MM2_X_MIN: i32 = -3678;
const MM2_X_MAX: i32 = 3934;
const MM2_Y_MIN: i32 = -2478;
const MM2_Y_MAX: i32 = 2587;
const PTP_MAX: i32 = 10000;

pub unsafe fn process_input_report(
    device: WDFDEVICE,
    raw_report: &MagicMouseInputReport,
    _len: usize,
) {
    let context = get_device_context(device);

    send_mouse_report((*context).report_queue, raw_report);

    if (*context).input_mode == PTPFeatureReport::MODE_PTP {
        send_ptp_report((*context).report_queue, raw_report);
    }
}

unsafe fn send_mouse_report(queue: WDFQUEUE, raw: &MagicMouseInputReport) {
    let mut request: WDFREQUEST = core::ptr::null_mut();
    let status = call_unsafe_wdf_function_binding!(
        WdfIoQueueRetrieveNextRequest, 
        queue, 
        &mut request
    );

    if status == STATUS_SUCCESS {
        let mut report = MouseInputReport {
            report_id: 1,
            buttons: raw.buttons,
            x: raw.dx,
            y: raw.dy,
        };

        let mut memory = core::ptr::null_mut();
        let mem_status = call_unsafe_wdf_function_binding!(
            WdfRequestRetrieveOutputMemory, 
            request, 
            &mut memory
        );
        
        if mem_status >= 0 {
            let _ = call_unsafe_wdf_function_binding!(
                WdfMemoryCopyFromBuffer,
                memory,
                0,
                &mut report as *mut _ as *mut c_void,
                core::mem::size_of::<MouseInputReport>()
            );
            call_unsafe_wdf_function_binding!(
                WdfRequestCompleteWithInformation,
                request,
                STATUS_SUCCESS,
                core::mem::size_of::<MouseInputReport>() as u64
            );
        } else {
            call_unsafe_wdf_function_binding!(
                WdfRequestCompleteWithInformation, 
                request, 
                STATUS_BUFFER_TOO_SMALL, 
                0
            );
        }
    }
}

unsafe fn send_ptp_report(queue: WDFQUEUE, raw: &MagicMouseInputReport) {
    let mut request: WDFREQUEST = core::ptr::null_mut();
    let status = call_unsafe_wdf_function_binding!(
        WdfIoQueueRetrieveNextRequest, 
        queue, 
        &mut request
    );

    if status == STATUS_SUCCESS {
        let mut ptp_report = PTPInputReport {
            report_id: 2,
            ..Default::default()
        };

        parse_mm2_touch(raw, &mut ptp_report);

        let mut memory = core::ptr::null_mut();
        let mem_status = call_unsafe_wdf_function_binding!(
            WdfRequestRetrieveOutputMemory, 
            request, 
            &mut memory
        );

        if mem_status >= 0 {
            let _ = call_unsafe_wdf_function_binding!(
                WdfMemoryCopyFromBuffer,
                memory,
                0,
                &mut ptp_report as *mut _ as *mut c_void,
                core::mem::size_of::<PTPInputReport>()
            );
            call_unsafe_wdf_function_binding!(
                WdfRequestCompleteWithInformation,
                request,
                STATUS_SUCCESS,
                core::mem::size_of::<PTPInputReport>() as u64
            );
        } else {
            call_unsafe_wdf_function_binding!(
                WdfRequestCompleteWithInformation, 
                request, 
                STATUS_BUFFER_TOO_SMALL, 
                0
            );
        }
    }
}

unsafe fn parse_mm2_touch(raw: &MagicMouseInputReport, ptp: &mut PTPInputReport) {
    let data = &raw.touch_data;
    let mut count = 0;

    for i in (0..data.len()).step_by(8) {
        if i + 8 > data.len() || count >= 5 {
            break;
        }

        let chunk = &data[i..i + 8];
        let state = chunk[3] & 0xF0;
        let id = (chunk[6] >> 2) & 0x0F;

        if state != 0 {
            let raw_x = (((chunk[1] as i32) << 28) | ((chunk[0] as i32) << 20)) >> 20;
            let raw_y = -((((chunk[3] as i32) << 30)
                | ((chunk[2] as i32) << 22)
                | ((chunk[1] as i32) << 14))
                >> 19);

            let ptp_x = scale(raw_x, MM2_X_MIN, MM2_X_MAX, PTP_MAX);
            let ptp_y = scale(raw_y, MM2_Y_MIN, MM2_Y_MAX, PTP_MAX);

            ptp.contacts[count] = PTPContact {
                status: 0x07,
                contact_id: id,
                x: ptp_x as u16,
                y: ptp_y as u16,
            };
            count += 1;
        }
    }
    ptp.contact_count = count as u8;
}

fn scale(val: i32, min: i32, max: i32, target: i32) -> i32 {
    let val = val.clamp(min, max);
    ((val - min) as i64 * target as i64 / (max - min) as i64) as i32
}