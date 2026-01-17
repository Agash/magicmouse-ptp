use crate::descriptors::MagicMouseInputReport;
use crate::input::process_input_report;
use crate::wdf_object_context::wdf_declare_context_type_with_name;
use wdk::println;
use wdk_sys::{
    call_unsafe_wdf_function_binding, NTSTATUS, STATUS_SUCCESS, WDFDEVICE, WDFREQUEST,
    WDF_POWER_DEVICE_STATE, WDF_REQUEST_COMPLETION_PARAMS, WDFIOTARGET,
};

pub struct DeviceContext {
    pub magic_sequence_sent: bool,
}

wdf_declare_context_type_with_name!(DeviceContext, get_device_context);

pub unsafe extern "C" fn evt_device_d0_entry(
    _device: WDFDEVICE,
    previous_state: WDF_POWER_DEVICE_STATE,
) -> NTSTATUS {
    println!("evt_device_d0_entry: Entry from state {:?}", previous_state);
    println!("evt_device_d0_entry: Completed successfully");
    STATUS_SUCCESS
}

pub unsafe extern "C" fn evt_device_d0_exit(
    _device: WDFDEVICE,
    target_state: WDF_POWER_DEVICE_STATE,
) -> NTSTATUS {
    println!("evt_device_d0_exit: Exiting to state {:?}", target_state);
    STATUS_SUCCESS
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
    
    call_unsafe_wdf_function_binding!(WdfRequestComplete, request, status);
}