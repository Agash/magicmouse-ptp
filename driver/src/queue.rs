use crate::descriptors::{HID_REPORT_DESCRIPTOR, PTPFeatureReport};
use crate::device::get_device_context;
use core::ffi::c_void;
use wdk_sys::{
    call_unsafe_wdf_function_binding,
    STATUS_INVALID_DEVICE_REQUEST, STATUS_SUCCESS, STATUS_BUFFER_TOO_SMALL,
    WDFQUEUE, WDFREQUEST, 
};

const IOCTL_HID_GET_DEVICE_ATTRIBUTES: u32 = 0xB0003;
const IOCTL_HID_GET_REPORT_DESCRIPTOR: u32 = 0xB0007;
const IOCTL_HID_READ_REPORT: u32 = 0xB001F;
const IOCTL_HID_SET_FEATURE: u32 = 0xB0191;
const IOCTL_HID_GET_FEATURE: u32 = 0xB0192;

// Correct signature: Queue, Request, OutputLen, InputLen, Code
pub unsafe extern "C" fn evt_io_internal_device_control(
    queue: WDFQUEUE,
    request: WDFREQUEST,
    _output_len: usize,
    _input_len: usize,
    ioctl_code: u32,
) {
    let device = call_unsafe_wdf_function_binding!(WdfIoQueueGetDevice, queue);
    let context = get_device_context(device);

    match ioctl_code {
        IOCTL_HID_GET_REPORT_DESCRIPTOR => {
            let mut memory = core::ptr::null_mut();
            let status = call_unsafe_wdf_function_binding!(
                WdfRequestRetrieveOutputMemory, 
                request, 
                &mut memory
            );
            if status >= 0 {
                let _ = call_unsafe_wdf_function_binding!(
                    WdfMemoryCopyFromBuffer,
                    memory,
                    0,
                    HID_REPORT_DESCRIPTOR.as_ptr() as *mut c_void,
                    HID_REPORT_DESCRIPTOR.len()
                );
                call_unsafe_wdf_function_binding!(
                    WdfRequestCompleteWithInformation,
                    request,
                    STATUS_SUCCESS,
                    HID_REPORT_DESCRIPTOR.len() as u64
                );
            } else {
                call_unsafe_wdf_function_binding!(
                    WdfRequestCompleteWithInformation, 
                    request, 
                    status, 
                    0
                );
            }
        }
        IOCTL_HID_GET_DEVICE_ATTRIBUTES => {
            call_unsafe_wdf_function_binding!(
                WdfRequestCompleteWithInformation, 
                request, 
                STATUS_SUCCESS, 
                0
            );
        }
        IOCTL_HID_READ_REPORT => {
            let status = call_unsafe_wdf_function_binding!(
                WdfRequestForwardToIoQueue, 
                request, 
                (*context).report_queue
            );
            if status < 0 {
                call_unsafe_wdf_function_binding!(
                    WdfRequestCompleteWithInformation, 
                    request, 
                    status, 
                    0
                );
            }
        }
        IOCTL_HID_SET_FEATURE => {
            let mut memory = core::ptr::null_mut();
            let status = call_unsafe_wdf_function_binding!(
                WdfRequestRetrieveInputMemory, 
                request, 
                &mut memory
            );
            if status == STATUS_SUCCESS {
                let mut buffer: [u8; 2] = [0; 2];
                let _ = call_unsafe_wdf_function_binding!(
                    WdfMemoryCopyToBuffer, 
                    memory, 
                    0, 
                    buffer.as_mut_ptr() as *mut c_void, 
                    2
                );
                
                if buffer[0] == PTPFeatureReport::REPORT_ID {
                    (*context).input_mode = buffer[1];
                    call_unsafe_wdf_function_binding!(
                        WdfRequestCompleteWithInformation, 
                        request, 
                        STATUS_SUCCESS, 
                        2
                    );
                    return;
                }
            }
            call_unsafe_wdf_function_binding!(
                WdfRequestCompleteWithInformation, 
                request, 
                STATUS_INVALID_DEVICE_REQUEST, 
                0
            );
        }
        IOCTL_HID_GET_FEATURE => {
             let mut memory = core::ptr::null_mut();
             let status = call_unsafe_wdf_function_binding!(
                 WdfRequestRetrieveOutputMemory, 
                 request, 
                 &mut memory
             );
             if status == STATUS_SUCCESS {
                 let report = PTPFeatureReport {
                     report_id: PTPFeatureReport::REPORT_ID,
                     input_mode: (*context).input_mode,
                 };
                 let _ = call_unsafe_wdf_function_binding!(
                     WdfMemoryCopyFromBuffer, 
                     memory, 
                     0, 
                     &report as *const _ as *mut c_void, 
                     2
                 );
                 call_unsafe_wdf_function_binding!(
                     WdfRequestCompleteWithInformation, 
                     request, 
                     STATUS_SUCCESS, 
                     2
                 );
                 return;
             }
             call_unsafe_wdf_function_binding!(
                 WdfRequestCompleteWithInformation, 
                 request, 
                 STATUS_BUFFER_TOO_SMALL, 
                 0
             );
        }
        _ => {
            call_unsafe_wdf_function_binding!(
                WdfRequestCompleteWithInformation, 
                request, 
                STATUS_INVALID_DEVICE_REQUEST, 
                0
            );
        }
    }
}