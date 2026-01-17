use crate::device::{evt_read_complete, get_device_context, send_magic_sequence};
use core::ffi::c_void;
use wdk::println;
use wdk_sys::{
    call_unsafe_wdf_function_binding, WDFQUEUE, WDFREQUEST, WDF_REQUEST_SEND_OPTIONS,
};

const MAGIC_SEQ_MM2: &[u8] = &[0xD7, 0x01];

pub unsafe extern "C" fn evt_io_device_control(
    queue: WDFQUEUE,
    request: WDFREQUEST,
    output_len: usize,
    input_len: usize,
    ioctl_code: u32,
) {
    println!(
        "evt_io_device_control: IOCTL {:#010X}, input {}, output {}",
        ioctl_code, input_len, output_len
    );
    
    // Try to send magic sequence on first IOCTL
    try_send_magic_sequence(queue);
    
    forward_request(queue, request, None);
}

pub unsafe extern "C" fn evt_io_internal_device_control(
    queue: WDFQUEUE,
    request: WDFREQUEST,
    output_len: usize,
    input_len: usize,
    ioctl_code: u32,
) {
    println!(
        "evt_io_internal_device_control: IOCTL {:#010X}, input {}, output {}",
        ioctl_code, input_len, output_len
    );
    forward_request(queue, request, None);
}

pub unsafe extern "C" fn evt_io_read(queue: WDFQUEUE, request: WDFREQUEST, length: usize) {
    println!("evt_io_read: Read request for {} bytes", length);
    
    let device = call_unsafe_wdf_function_binding!(WdfIoQueueGetDevice, queue);
    
    // Try to send magic sequence on first read
    try_send_magic_sequence(queue);
    
    // Set completion routine to intercept the response
    call_unsafe_wdf_function_binding!(
        WdfRequestSetCompletionRoutine,
        request,
        Some(evt_read_complete),
        device as *mut c_void
    );
    
    forward_request(queue, request, None);
}

unsafe fn try_send_magic_sequence(queue: WDFQUEUE) {
    let device = call_unsafe_wdf_function_binding!(WdfIoQueueGetDevice, queue);
    let device_context = get_device_context(device as wdk_sys::WDFOBJECT);
    
    if device_context.is_null() {
        return;
    }
    
    // Only send once
    if (*device_context).magic_sequence_sent {
        return;
    }
    
    if !(*device_context).io_target_started {
        println!("try_send_magic_sequence: I/O target not ready yet");
        return;
    }
    
    println!("try_send_magic_sequence: Sending magic sequence now");
    (*device_context).magic_sequence_sent = true;
    
    let status = send_magic_sequence(device, MAGIC_SEQ_MM2);
    if status < 0 {
        println!("try_send_magic_sequence: Failed with status {:#010X}", status);
        // Don't retry - just log and continue
    } else {
        println!("try_send_magic_sequence: Success!");
    }
}

unsafe fn forward_request(
    queue: WDFQUEUE,
    request: WDFREQUEST,
    options: Option<*mut WDF_REQUEST_SEND_OPTIONS>,
) {
    let device = call_unsafe_wdf_function_binding!(WdfIoQueueGetDevice, queue);
    let target = call_unsafe_wdf_function_binding!(WdfDeviceGetIoTarget, device);
    
    if target.is_null() {
        println!("forward_request: I/O target is null, completing with error");
        call_unsafe_wdf_function_binding!(
            WdfRequestComplete,
            request,
            wdk_sys::STATUS_INVALID_DEVICE_STATE
        );
        return;
    }
    
    // Format the request using the current type (maintains original IRP structure)
    call_unsafe_wdf_function_binding!(WdfRequestFormatRequestUsingCurrentType, request);
    
    // Send the request down the stack
    let send_status = call_unsafe_wdf_function_binding!(
        WdfRequestSend,
        request,
        target,
        options.unwrap_or(core::ptr::null_mut())
    );
    
    if send_status == 0 {
        // WdfRequestSend returns FALSE (0) on failure
        let status = call_unsafe_wdf_function_binding!(WdfRequestGetStatus, request);
        println!("forward_request: WdfRequestSend failed with status {:#010X}", status);
        call_unsafe_wdf_function_binding!(WdfRequestComplete, request, status);
    }
    // If send_status is TRUE (1), the request is now owned by the lower driver
}