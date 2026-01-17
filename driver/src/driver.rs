use crate::device::{evt_device_d0_entry, DeviceContext, WDF_DEVICE_CONTEXT_TYPE_INFO};
use crate::queue::{evt_io_device_control, evt_io_read};
use crate::wdf_object_context::wdf_get_context_type_info;
use wdk_sys::{
    call_unsafe_wdf_function_binding, NTSTATUS, WDFDEVICE, WDFDRIVER, WDFQUEUE,
    WDF_IO_QUEUE_CONFIG, WDF_NO_HANDLE, WDF_NO_OBJECT_ATTRIBUTES, WDF_OBJECT_ATTRIBUTES,
    WDF_PNPPOWER_EVENT_CALLBACKS, _WDF_IO_QUEUE_DISPATCH_TYPE,
};

pub unsafe extern "C" fn evt_device_add(
    _driver: WDFDRIVER,
    mut device_init: *mut wdk_sys::WDFDEVICE_INIT,
) -> NTSTATUS {
    // CRITICAL: Configure as a Filter Driver.
    call_unsafe_wdf_function_binding!(WdfFdoInitSetFilter, device_init);

    // Set PnP/Power Callbacks (D0Entry)
    let mut pnp_power_callbacks = WDF_PNPPOWER_EVENT_CALLBACKS::default();
    pnp_power_callbacks.Size = core::mem::size_of::<WDF_PNPPOWER_EVENT_CALLBACKS>() as u32;
    pnp_power_callbacks.EvtDeviceD0Entry = Some(evt_device_d0_entry);
    call_unsafe_wdf_function_binding!(
        WdfDeviceInitSetPnpPowerEventCallbacks,
        device_init,
        &mut pnp_power_callbacks
    );

    // Create Device
    let mut attributes = WDF_OBJECT_ATTRIBUTES::default();
    attributes.Size = core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as u32;
    attributes.ContextTypeInfo = wdf_get_context_type_info!(DeviceContext);

    let mut device: WDFDEVICE = WDF_NO_HANDLE as WDFDEVICE;
    let status = call_unsafe_wdf_function_binding!(
        WdfDeviceCreate,
        &mut device_init,
        &mut attributes,
        &mut device
    );
    if status < 0 {
        return status;
    }

    // Default Queue: Forwarding
    let mut queue_config = WDF_IO_QUEUE_CONFIG::default();
    queue_config.Size = core::mem::size_of::<WDF_IO_QUEUE_CONFIG>() as u32;
    queue_config.DispatchType = _WDF_IO_QUEUE_DISPATCH_TYPE::WdfIoQueueDispatchParallel;

    // Intercept Reads (Sniffing)
    queue_config.EvtIoRead = Some(evt_io_read);
    // Forward IOCTLs
    queue_config.EvtIoDeviceControl = Some(evt_io_device_control);
    queue_config.EvtIoInternalDeviceControl = Some(evt_io_device_control);

    let mut queue: WDFQUEUE = WDF_NO_HANDLE as WDFQUEUE;
    call_unsafe_wdf_function_binding!(
        WdfIoQueueCreate,
        device,
        &mut queue_config,
        WDF_NO_OBJECT_ATTRIBUTES,
        &mut queue
    )
}