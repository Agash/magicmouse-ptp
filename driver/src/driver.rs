use crate::device::{evt_device_d0_entry, evt_device_d0_exit, get_device_context, WDF_DEVICE_CONTEXT_TYPE_INFO};
use crate::queue::{evt_io_device_control, evt_io_internal_device_control, evt_io_read};
use crate::wdf_object_context::wdf_get_context_type_info;
use wdk::println;
use wdk_sys::{
    call_unsafe_wdf_function_binding, NTSTATUS, STATUS_SUCCESS, WDFDEVICE, WDFDRIVER, WDFQUEUE,
    WDF_IO_QUEUE_CONFIG, WDF_NO_HANDLE, WDF_OBJECT_ATTRIBUTES,
    WDF_PNPPOWER_EVENT_CALLBACKS, _WDF_IO_QUEUE_DISPATCH_TYPE, _WDF_EXECUTION_LEVEL,
    _WDF_SYNCHRONIZATION_SCOPE,
};

pub unsafe extern "C" fn evt_device_add(
    _driver: WDFDRIVER,
    mut device_init: *mut wdk_sys::WDFDEVICE_INIT,
) -> NTSTATUS {
    println!("evt_device_add: Starting device initialization");
    
    // Configure as a filter driver
    call_unsafe_wdf_function_binding!(WdfFdoInitSetFilter, device_init);
    
    // Set up PnP/Power callbacks
    let mut pnp_power_callbacks = WDF_PNPPOWER_EVENT_CALLBACKS::default();
    pnp_power_callbacks.Size = core::mem::size_of::<WDF_PNPPOWER_EVENT_CALLBACKS>() as u32;
    pnp_power_callbacks.EvtDeviceD0Entry = Some(evt_device_d0_entry);
    pnp_power_callbacks.EvtDeviceD0Exit = Some(evt_device_d0_exit);
    
    call_unsafe_wdf_function_binding!(
        WdfDeviceInitSetPnpPowerEventCallbacks,
        device_init,
        &mut pnp_power_callbacks
    );
    
    // Set up device context
    let mut attributes = WDF_OBJECT_ATTRIBUTES::default();
    attributes.Size = core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as u32;
    attributes.ExecutionLevel = _WDF_EXECUTION_LEVEL::WdfExecutionLevelInheritFromParent;
    attributes.SynchronizationScope = _WDF_SYNCHRONIZATION_SCOPE::WdfSynchronizationScopeInheritFromParent;
    attributes.ContextTypeInfo = wdf_get_context_type_info!(DeviceContext);
    
    let mut device: WDFDEVICE = WDF_NO_HANDLE as WDFDEVICE;
    let mut status = call_unsafe_wdf_function_binding!(
        WdfDeviceCreate,
        &mut device_init,
        &mut attributes,
        &mut device
    );
    
    if status < 0 {
        println!("evt_device_add: WdfDeviceCreate failed with status {:#010X}", status);
        return status;
    }
    
    // Initialize device context
    let device_context = get_device_context(device as wdk_sys::WDFOBJECT);
    if device_context.is_null() {
        println!("evt_device_add: Failed to get device context");
        return wdk_sys::STATUS_INSUFFICIENT_RESOURCES;
    }
    (*device_context).io_target_started = false;
    (*device_context).magic_sequence_sent = false;
    
    println!("evt_device_add: Device created successfully, creating queues");
    
    // Create a parallel queue for IOCTLs and reads
    let mut queue_config = WDF_IO_QUEUE_CONFIG::default();
    queue_config.Size = core::mem::size_of::<WDF_IO_QUEUE_CONFIG>() as u32;
    queue_config.DispatchType = _WDF_IO_QUEUE_DISPATCH_TYPE::WdfIoQueueDispatchParallel;
    queue_config.EvtIoRead = Some(evt_io_read);
    queue_config.EvtIoDeviceControl = Some(evt_io_device_control);
    queue_config.EvtIoInternalDeviceControl = Some(evt_io_internal_device_control);
    queue_config.PowerManaged = wdk_sys::_WDF_TRI_STATE::WdfTrue;
    queue_config.DefaultQueue = 1; // This is the default queue
    
    // For filter drivers, we must inherit execution level from parent
    let mut attributes = WDF_OBJECT_ATTRIBUTES::default();
    attributes.Size = core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as u32;
    attributes.ExecutionLevel = _WDF_EXECUTION_LEVEL::WdfExecutionLevelInheritFromParent;
    attributes.SynchronizationScope = _WDF_SYNCHRONIZATION_SCOPE::WdfSynchronizationScopeInheritFromParent;
    
    let mut queue: WDFQUEUE = WDF_NO_HANDLE as WDFQUEUE;
    status = call_unsafe_wdf_function_binding!(
        WdfIoQueueCreate,
        device,
        &mut queue_config,
        &mut attributes,
        &mut queue
    );
    
    if status < 0 {
        println!("evt_device_add: WdfIoQueueCreate failed with status {:#010X}", status);
        return status;
    }
    
    println!("evt_device_add: Queue created successfully");
    
    STATUS_SUCCESS
}