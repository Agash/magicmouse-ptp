#![no_std]
#![allow(non_snake_case)]

extern crate wdk_alloc;

#[cfg(not(test))]
extern crate wdk_panic;

#[global_allocator]
static ALLOCATOR: wdk_alloc::WdkAllocator = wdk_alloc::WdkAllocator;

use wdk_sys::{
    call_unsafe_wdf_function_binding,
    NTSTATUS, PDRIVER_OBJECT, PUNICODE_STRING, WDF_DRIVER_CONFIG,
    WDF_NO_OBJECT_ATTRIBUTES, WDF_NO_HANDLE, WDFDRIVER,
};

mod descriptors;
mod device;
mod driver;
mod input;
mod queue;

use driver::evt_device_add;

#[export_name = "DriverEntry"]
pub unsafe extern "system" fn driver_entry(
    driver_object: PDRIVER_OBJECT,
    registry_path: PUNICODE_STRING,
) -> NTSTATUS {
    let mut config = WDF_DRIVER_CONFIG::default();
    config.Size = core::mem::size_of::<WDF_DRIVER_CONFIG>() as u32;
    config.EvtDriverDeviceAdd = Some(evt_device_add);

    let mut driver_handle = WDF_NO_HANDLE as WDFDRIVER;

    let status = call_unsafe_wdf_function_binding!(
        WdfDriverCreate,
        driver_object,
        registry_path,
        WDF_NO_OBJECT_ATTRIBUTES,
        &mut config,
        &mut driver_handle
    );

    status
}