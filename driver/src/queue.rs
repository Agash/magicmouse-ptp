use crate::device::evt_read_complete;
use core::ffi::c_void;
use wdk_sys::{
    call_unsafe_wdf_function_binding, WDFQUEUE, WDFREQUEST
};

pub unsafe extern "C" fn evt_io_device_control(
    queue: WDFQUEUE,
    request: WDFREQUEST,
    _output_len: usize,
    _input_len: usize,
    _ioctl_code: u32,
) {
    forward_request(queue, request);
}

pub unsafe extern "C" fn evt_io_read(queue: WDFQUEUE, request: WDFREQUEST, _length: usize) {
    let device = call_unsafe_wdf_function_binding!(WdfIoQueueGetDevice, queue);

    // Attach Sniffer Completion Routine
    call_unsafe_wdf_function_binding!(
        WdfRequestSetCompletionRoutine,
        request,
        Some(evt_read_complete),
        device as *mut c_void
    );

    forward_request(queue, request);
}

unsafe fn forward_request(queue: WDFQUEUE, request: WDFREQUEST) {
    let device = call_unsafe_wdf_function_binding!(WdfIoQueueGetDevice, queue);
    let target = call_unsafe_wdf_function_binding!(WdfDeviceGetIoTarget, device);

    // Format for next driver
    call_unsafe_wdf_function_binding!(WdfRequestFormatRequestUsingCurrentType, request);

    // Send (Fire and Forget)
    let sent = call_unsafe_wdf_function_binding!(
        WdfRequestSend,
        request,
        target,
        core::ptr::null_mut()
    );

    if sent == 0 {
        let status = call_unsafe_wdf_function_binding!(WdfRequestGetStatus, request);
        call_unsafe_wdf_function_binding!(WdfRequestComplete, request, status);
    }
}