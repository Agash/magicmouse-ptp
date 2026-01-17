use crate::descriptors::MagicMouseInputReport;
use crate::input::process_input_report;
use crate::wdf_object_context::wdf_declare_context_type_with_name;
use core::ffi::c_void;
use wdk::println;
use wdk_sys::{
    call_unsafe_wdf_function_binding, NTSTATUS, STATUS_SUCCESS, WDFDEVICE, WDFREQUEST,
    WDF_MEMORY_DESCRIPTOR, WDF_POWER_DEVICE_STATE, WDF_REQUEST_COMPLETION_PARAMS, WDFIOTARGET,
    _WDF_MEMORY_DESCRIPTOR_TYPE,
};

pub struct DeviceContext {
    pub io_target_started: bool,
    pub magic_sequence_sent: bool,
}

wdf_declare_context_type_with_name!(DeviceContext, get_device_context);

const IOCTL_HID_SET_FEATURE: u32 = 0xB0191;
const MAGIC_SEQ_MM2: &[u8] = &[0xD7, 0x01];

pub unsafe extern "C" fn evt_device_d0_entry(
    device: WDFDEVICE,
    previous_state: WDF_POWER_DEVICE_STATE,
) -> NTSTATUS {
    println!("evt_device_d0_entry: Entry from state {:?}", previous_state);
    
    let device_context = get_device_context(device as wdk_sys::WDFOBJECT);
    if device_context.is_null() {
        println!("evt_device_d0_entry: Failed to get device context");
        return STATUS_SUCCESS;
    }
    
    // Mark that the I/O target is now ready
    (*device_context).io_target_started = true;
    
    // DON'T send magic sequence here - it causes a deadlock!
    // The device is still initializing and can't process IOCTLs yet.
    // We'll send it on the first read request instead.
    
    println!("evt_device_d0_entry: D0Entry complete, will send magic sequence on first I/O");
    
    STATUS_SUCCESS
}

pub unsafe extern "C" fn evt_device_d0_exit(
    device: WDFDEVICE,
    target_state: WDF_POWER_DEVICE_STATE,
) -> NTSTATUS {
    println!("evt_device_d0_exit: Exiting to state {:?}", target_state);
    
    let device_context = get_device_context(device as wdk_sys::WDFOBJECT);
    if !device_context.is_null() {
        (*device_context).io_target_started = false;
    }
    
    STATUS_SUCCESS
}

pub unsafe fn send_magic_sequence(device: WDFDEVICE, sequence: &[u8]) -> NTSTATUS {
    let io_target = call_unsafe_wdf_function_binding!(WdfDeviceGetIoTarget, device);
    if io_target.is_null() {
        println!("send_magic_sequence: I/O target is null");
        return wdk_sys::STATUS_INVALID_DEVICE_STATE;
    }
    
    let mut mem_desc = WDF_MEMORY_DESCRIPTOR::default();
    mem_desc.Type = _WDF_MEMORY_DESCRIPTOR_TYPE::WdfMemoryDescriptorTypeBuffer;
    mem_desc.u.BufferType.Buffer = sequence.as_ptr() as *mut c_void;
    mem_desc.u.BufferType.Length = sequence.len() as u32;
    
    println!("send_magic_sequence: Sending {} bytes to I/O target", sequence.len());
    
    call_unsafe_wdf_function_binding!(
        WdfIoTargetSendInternalIoctlSynchronously,
        io_target,
        core::ptr::null_mut(),
        IOCTL_HID_SET_FEATURE,
        &mut mem_desc,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut()
    )
}

pub unsafe extern "C" fn evt_read_complete(
    request: WDFREQUEST,
    _target: WDFIOTARGET,
    params: *mut WDF_REQUEST_COMPLETION_PARAMS,
    context: wdk_sys::WDFCONTEXT,
) {
    let device = context as WDFDEVICE;
    
    if params.is_null() {
        println!("evt_read_complete: params is null");
        call_unsafe_wdf_function_binding!(
            WdfRequestComplete,
            request,
            wdk_sys::STATUS_INVALID_PARAMETER
        );
        return;
    }
    
    let status = (*params).IoStatus.__bindgen_anon_1.Status;
    
    if status != STATUS_SUCCESS {
        // Just forward the request with its status
        call_unsafe_wdf_function_binding!(WdfRequestComplete, request, status);
        return;
    }
    
    let mut memory = core::ptr::null_mut();
    let mem_status = call_unsafe_wdf_function_binding!(
        WdfRequestRetrieveOutputMemory,
        request,
        &mut memory
    );
    
    if mem_status < 0 {
        println!("evt_read_complete: Failed to retrieve memory: {:#010X}", mem_status);
        call_unsafe_wdf_function_binding!(WdfRequestComplete, request, mem_status);
        return;
    }
    
    let mut buffer_size: usize = 0;
    let buffer_ptr = call_unsafe_wdf_function_binding!(
        WdfMemoryGetBuffer,
        memory,
        &mut buffer_size as *mut usize as *mut _
    );
    
    if buffer_ptr.is_null() {
        println!("evt_read_complete: Buffer pointer is null");
        call_unsafe_wdf_function_binding!(WdfRequestComplete, request, status);
        return;
    }
    
    let len = (*params).IoStatus.Information as usize;
    if len < 1 || len > buffer_size {
        println!("evt_read_complete: Invalid length {} (buffer size {})", len, buffer_size);
        call_unsafe_wdf_function_binding!(WdfRequestComplete, request, status);
        return;
    }
    
    let report_id = *(buffer_ptr as *const u8);
    println!("evt_read_complete: Report ID {:#04X}, length {}", report_id, len);
    
    if report_id == MagicMouseInputReport::REPORT_ID && len >= core::mem::size_of::<MagicMouseInputReport>() {
        let raw = &*(buffer_ptr as *const MagicMouseInputReport);
        process_input_report(device, raw, len);
    }
    
    // Complete the request with the original status
    call_unsafe_wdf_function_binding!(WdfRequestComplete, request, status);
}