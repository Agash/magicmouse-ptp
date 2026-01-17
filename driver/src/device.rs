use crate::descriptors::MagicMouseInputReport;
use crate::input::process_input_report;
use crate::wdf_object_context::wdf_declare_context_type_with_name;
use core::ffi::c_void;
use wdk_sys::{
    call_unsafe_wdf_function_binding, NTSTATUS, STATUS_SUCCESS, WDFDEVICE, WDFREQUEST,
    WDF_MEMORY_DESCRIPTOR, WDF_POWER_DEVICE_STATE, WDF_REQUEST_COMPLETION_PARAMS, WDFIOTARGET,
    _WDF_MEMORY_DESCRIPTOR_TYPE,
};

// Define Context with Safe Helper
pub struct DeviceContext {
    pub dummy: u32,
}
wdf_declare_context_type_with_name!(DeviceContext, get_device_context);

const IOCTL_HID_SET_FEATURE: u32 = 0xB0191;
const MAGIC_SEQ_MM2: &[u8] = &[0xD7, 0x01];

pub unsafe extern "C" fn evt_device_d0_entry(
    device: WDFDEVICE,
    _previous_state: WDF_POWER_DEVICE_STATE,
) -> NTSTATUS {
    let _ = send_magic_sequence(device, MAGIC_SEQ_MM2);
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

pub unsafe extern "C" fn evt_read_complete(
    request: WDFREQUEST,
    _target: WDFIOTARGET,
    params: *mut WDF_REQUEST_COMPLETION_PARAMS,
    context: wdk_sys::WDFCONTEXT,
) {
    let device = context as WDFDEVICE;
    let status = (*params).IoStatus.__bindgen_anon_1.Status;

    if status == STATUS_SUCCESS {
        let mut memory = core::ptr::null_mut();
        let mem_status = call_unsafe_wdf_function_binding!(
            WdfRequestRetrieveOutputMemory,
            request,
            &mut memory
        );

        if mem_status >= 0 {
            let buffer_ptr = call_unsafe_wdf_function_binding!(
                WdfMemoryGetBuffer,
                memory,
                core::ptr::null_mut()
            );
            
            let len = (*params).IoStatus.Information as usize;

            if !buffer_ptr.is_null() && len >= 1 {
                let report_id = *(buffer_ptr as *const u8);
                if report_id == MagicMouseInputReport::REPORT_ID {
                     let raw = &*(buffer_ptr as *const MagicMouseInputReport);
                     process_input_report(device, raw, len);
                }
            }
        }
    }
    
    call_unsafe_wdf_function_binding!(WdfRequestComplete, request, status);
}