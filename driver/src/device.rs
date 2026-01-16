use core::ffi::c_void;
use wdk_sys::{
    call_unsafe_wdf_function_binding,
    NTSTATUS, STATUS_SUCCESS, WDFDEVICE, WDFMEMORY, WDFQUEUE, WDFREQUEST,
    WDF_MEMORY_DESCRIPTOR, WDF_POWER_DEVICE_STATE,
    WDF_REQUEST_REUSE_PARAMS, 
    _WDF_REQUEST_REUSE_FLAGS, _WDF_MEMORY_DESCRIPTOR_TYPE
};
use crate::input::process_input_report;
use crate::descriptors::MagicMouseInputReport;

const IOCTL_HID_READ_REPORT: u32 = 0xB001F;
const IOCTL_HID_SET_FEATURE: u32 = 0xB0191;
const MAGIC_SEQ_MM1: &[u8] = &[0xD7, 0x01];
const MAGIC_SEQ_MM2: &[u8] = &[0xF1, 0x02, 0x01];

#[repr(C)]
pub struct DeviceContext {
    pub report_queue: WDFQUEUE,
    pub read_request: WDFREQUEST,
    pub read_memory: WDFMEMORY,
    pub input_mode: u8,
}

impl Default for DeviceContext {
    fn default() -> Self {
        Self {
            report_queue: core::ptr::null_mut(),
            read_request: core::ptr::null_mut(),
            read_memory: core::ptr::null_mut(),
            input_mode: 0x03,
        }
    }
}

pub unsafe fn get_device_context(device: WDFDEVICE) -> *mut DeviceContext {
    let context_ptr = call_unsafe_wdf_function_binding!(
        WdfObjectGetTypedContextWorker,
        device as *mut _,
        core::ptr::null_mut()
    );
    context_ptr as *mut DeviceContext
}

pub unsafe extern "C" fn evt_device_d0_entry(
    device: WDFDEVICE,
    _previous_state: WDF_POWER_DEVICE_STATE,
) -> NTSTATUS {
    if send_magic_sequence(device, MAGIC_SEQ_MM2) < 0 {
        let _ = send_magic_sequence(device, MAGIC_SEQ_MM1);
    }
    start_continuous_reader(device);
    STATUS_SUCCESS
}

unsafe fn send_magic_sequence(device: WDFDEVICE, sequence: &[u8]) -> NTSTATUS {
    let io_target = call_unsafe_wdf_function_binding!(WdfDeviceGetIoTarget, device);
    let mut mem_desc = WDF_MEMORY_DESCRIPTOR::default();
    
    mem_desc.Type = _WDF_MEMORY_DESCRIPTOR_TYPE::WdfMemoryDescriptorTypeBuffer;
    mem_desc.u.BufferType.Buffer = sequence.as_ptr() as *mut c_void;
    mem_desc.u.BufferType.Length = sequence.len() as u32;

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

pub unsafe fn start_continuous_reader(device: WDFDEVICE) {
    let context = get_device_context(device);
    let target = call_unsafe_wdf_function_binding!(WdfDeviceGetIoTarget, device);
    let request = (*context).read_request;
    let memory = (*context).read_memory;

    let mut reuse_params = WDF_REQUEST_REUSE_PARAMS::default();
    reuse_params.Size = core::mem::size_of::<WDF_REQUEST_REUSE_PARAMS>() as u32;
    reuse_params.Flags = _WDF_REQUEST_REUSE_FLAGS::WDF_REQUEST_REUSE_NO_FLAGS as u32;
    reuse_params.Status = STATUS_SUCCESS;

    let _ = call_unsafe_wdf_function_binding!(
        WdfRequestReuse, 
        request, 
        &mut reuse_params
    );

    let status = call_unsafe_wdf_function_binding!(
        WdfIoTargetFormatRequestForInternalIoctl,
        target,
        request,
        IOCTL_HID_READ_REPORT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        memory,
        core::ptr::null_mut()
    );

    if status >= 0 {
        call_unsafe_wdf_function_binding!(
            WdfRequestSetCompletionRoutine,
            request,
            Some(evt_read_complete),
            device as *mut c_void
        );
        let _ = call_unsafe_wdf_function_binding!(
            WdfRequestSend, 
            request, 
            target, 
            core::ptr::null_mut()
        );
    }
}

unsafe extern "C" fn evt_read_complete(
    _request: WDFREQUEST,
    _target: wdk_sys::WDFIOTARGET,
    params: *mut wdk_sys::WDF_REQUEST_COMPLETION_PARAMS,
    context: wdk_sys::WDFCONTEXT,
) {
    let device = context as WDFDEVICE;
    let dev_ctx = get_device_context(device);
    
    let status = (*params).IoStatus.__bindgen_anon_1.Status;

    if status == STATUS_SUCCESS {
        let buffer_len = (*params).IoStatus.Information as usize;
        let mem_ptr = call_unsafe_wdf_function_binding!(
            WdfMemoryGetBuffer, 
            (*dev_ctx).read_memory, 
            core::ptr::null_mut()
        );

        if !mem_ptr.is_null() && buffer_len >= 8 {
            let report_id = *(mem_ptr as *const u8);
            if report_id == MagicMouseInputReport::REPORT_ID {
                let raw_report = &*(mem_ptr as *const MagicMouseInputReport);
                process_input_report(device, raw_report, buffer_len);
            }
        }
    }
    start_continuous_reader(device);
}