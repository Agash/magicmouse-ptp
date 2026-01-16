use wdk_sys::{
    call_unsafe_wdf_function_binding,
    NTSTATUS, STATUS_SUCCESS, WDFDEVICE, WDFDRIVER, WDF_IO_QUEUE_CONFIG,
    WDF_NO_OBJECT_ATTRIBUTES, WDF_OBJECT_ATTRIBUTES,
    WDF_PNPPOWER_EVENT_CALLBACKS, 
    _WDF_IO_QUEUE_DISPATCH_TYPE, _WDF_DEVICE_IO_TYPE, _POOL_TYPE,
    WDF_NO_HANDLE, WDFQUEUE, WDFMEMORY, WDFREQUEST,
};

use crate::device::{evt_device_d0_entry, get_device_context, DeviceContext};
use crate::queue::evt_io_internal_device_control;

pub unsafe extern "C" fn evt_device_add(
    _driver: WDFDRIVER,
    mut device_init: *mut wdk_sys::WDFDEVICE_INIT, // Fix: Added 'mut' to allow mutable borrow
) -> NTSTATUS {
    call_unsafe_wdf_function_binding!(
        WdfDeviceInitSetIoType,
        device_init, 
        _WDF_DEVICE_IO_TYPE::WdfDeviceIoDirect
    );

    let mut pnp_power_callbacks = WDF_PNPPOWER_EVENT_CALLBACKS::default();
    pnp_power_callbacks.Size = core::mem::size_of::<WDF_PNPPOWER_EVENT_CALLBACKS>() as u32;
    pnp_power_callbacks.EvtDeviceD0Entry = Some(evt_device_d0_entry);
    
    call_unsafe_wdf_function_binding!(
        WdfDeviceInitSetPnpPowerEventCallbacks,
        device_init, 
        &mut pnp_power_callbacks
    );

    let mut attributes = WDF_OBJECT_ATTRIBUTES::default();
    attributes.Size = core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as u32;
    attributes.ContextSizeOverride = core::mem::size_of::<DeviceContext>() as usize;

    let mut device: WDFDEVICE = WDF_NO_HANDLE as WDFDEVICE;
    
    // device_init is now a mutable local variable containing a pointer
    let status = call_unsafe_wdf_function_binding!(
        WdfDeviceCreate,
        &mut device_init, 
        &mut attributes, 
        &mut device
    );
    if status < 0 { return status; }

    let context = get_device_context(device);

    // 1. Manual Queue
    let mut manual_queue_config = WDF_IO_QUEUE_CONFIG::default();
    manual_queue_config.Size = core::mem::size_of::<WDF_IO_QUEUE_CONFIG>() as u32;
    manual_queue_config.DispatchType = _WDF_IO_QUEUE_DISPATCH_TYPE::WdfIoQueueDispatchManual;

    let mut manual_queue: WDFQUEUE = WDF_NO_HANDLE as WDFQUEUE;
    let status = call_unsafe_wdf_function_binding!(
        WdfIoQueueCreate,
        device,
        &mut manual_queue_config,
        WDF_NO_OBJECT_ATTRIBUTES,
        &mut manual_queue
    );
    if status < 0 { return status; }
    (*context).report_queue = manual_queue;

    // 2. Default Queue
    let mut default_queue_config = WDF_IO_QUEUE_CONFIG::default();
    default_queue_config.Size = core::mem::size_of::<WDF_IO_QUEUE_CONFIG>() as u32;
    default_queue_config.DispatchType = _WDF_IO_QUEUE_DISPATCH_TYPE::WdfIoQueueDispatchParallel;
    default_queue_config.EvtIoInternalDeviceControl = Some(evt_io_internal_device_control);

    let mut default_queue: WDFQUEUE = WDF_NO_HANDLE as WDFQUEUE;
    let status = call_unsafe_wdf_function_binding!(
        WdfIoQueueCreate,
        device,
        &mut default_queue_config,
        WDF_NO_OBJECT_ATTRIBUTES,
        &mut default_queue
    );
    if status < 0 { return status; }

    // 3. Pre-allocate Memory
    let mut read_mem_attr = WDF_OBJECT_ATTRIBUTES::default();
    read_mem_attr.Size = core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as u32;
    read_mem_attr.ParentObject = device as *mut _;
    
    let mut read_memory: WDFMEMORY = WDF_NO_HANDLE as WDFMEMORY;
    let status = call_unsafe_wdf_function_binding!(
        WdfMemoryCreate,
        &mut read_mem_attr,
        _POOL_TYPE::PagedPool,
        0,
        128,
        &mut read_memory,
        core::ptr::null_mut()
    );
    if status < 0 { return status; }
    (*context).read_memory = read_memory;

    // 4. Pre-allocate Request
    let mut req_attr = WDF_OBJECT_ATTRIBUTES::default();
    req_attr.Size = core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as u32;
    req_attr.ParentObject = device as *mut _;
    
    let mut read_request: WDFREQUEST = WDF_NO_HANDLE as WDFREQUEST;
    let io_target = call_unsafe_wdf_function_binding!(WdfDeviceGetIoTarget, device);
    
    let status = call_unsafe_wdf_function_binding!(
        WdfRequestCreate,
        &mut req_attr,
        io_target,
        &mut read_request
    );
    if status < 0 { return status; }
    (*context).read_request = read_request;

    STATUS_SUCCESS
}