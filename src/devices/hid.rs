use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use web_sys::{
    Hid, HidConnectionEvent, HidDevice, HidDeviceFilter, HidDeviceRequestOptions,
    HidInputReportEvent,
};

use super::{DeviceError, navigator, navigator_has};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceFilter {
    pub vendor_id: u16,
    pub product_id: Option<u16>,
    pub usage_page: Option<u16>,
    pub usage: Option<u16>,
}

pub mod anagram {
    use super::DeviceFilter;
    use crate::devices::anagram::{USB_PRODUCT_ID, USB_VENDOR_ID};

    pub const FILTER: DeviceFilter = DeviceFilter {
        vendor_id: USB_VENDOR_ID,
        product_id: Some(USB_PRODUCT_ID),
        usage_page: Some(0x00FF),
        usage: Some(0x01),
    };
    pub const REPORT_ID: u8 = 0;
}

impl From<DeviceFilter> for HidDeviceFilter {
    fn from(filter: DeviceFilter) -> Self {
        let hid_filter = Self::new();
        hid_filter.set_vendor_id(filter.vendor_id.into());
        filter
            .product_id
            .into_iter()
            .for_each(|id| hid_filter.set_product_id(id));
        filter
            .usage_page
            .into_iter()
            .for_each(|page| hid_filter.set_usage_page(page));
        filter
            .usage
            .into_iter()
            .for_each(|usage| hid_filter.set_usage(usage));
        hid_filter
    }
}

#[must_use]
pub fn is_supported() -> bool {
    navigator_has("hid")
}

fn hid() -> Result<Hid, DeviceError> {
    is_supported()
        .then(|| navigator().hid())
        .ok_or(DeviceError::Unsupported("WebHID"))
}

pub async fn request_devices(filters: &[DeviceFilter]) -> Result<Vec<HidDevice>, DeviceError> {
    let hid_filters: Vec<HidDeviceFilter> = filters.iter().copied().map(Into::into).collect();
    let options = HidDeviceRequestOptions::new(&hid_filters);
    Ok(hid()?.request_device(&options).await?.into_iter().collect())
}

pub async fn granted_devices() -> Result<Vec<HidDevice>, DeviceError> {
    Ok(hid()?.get_devices().await?.into_iter().collect())
}

pub async fn open(device: &HidDevice) -> Result<(), DeviceError> {
    if !device.opened() {
        device.open().await?;
    }
    Ok(())
}

pub async fn send_report(
    device: &HidDevice,
    report_id: u8,
    data: &[u8],
) -> Result<(), DeviceError> {
    device
        .send_report_with_u8_array(report_id, &js_sys::Uint8Array::from(data))?
        .await?;
    Ok(())
}

pub fn on_input_report(
    device: &HidDevice,
    mut callback: impl FnMut(u8, Vec<u8>) + 'static,
) -> EventListener {
    EventListener::new(device, "inputreport", move |event| {
        let report = event.unchecked_ref::<HidInputReportEvent>();
        callback(report.report_id(), data_view_bytes(&report.data()));
    })
}

pub fn on_connect(callback: impl Fn() + 'static) -> Result<EventListener, DeviceError> {
    Ok(EventListener::new(&hid()?.into(), "connect", move |_| {
        callback();
    }))
}

pub fn on_disconnect(callback: impl Fn(HidDevice) + 'static) -> Result<EventListener, DeviceError> {
    Ok(EventListener::new(
        &hid()?.into(),
        "disconnect",
        move |event| {
            callback(event.unchecked_ref::<HidConnectionEvent>().device());
        },
    ))
}

fn data_view_bytes(view: &js_sys::DataView) -> Vec<u8> {
    let byte_offset = u32::try_from(view.byte_offset()).unwrap_or(u32::MAX);
    let byte_length = u32::try_from(view.byte_length()).unwrap_or(u32::MAX);
    js_sys::Uint8Array::new_with_byte_offset_and_length(&view.buffer(), byte_offset, byte_length)
        .to_vec()
}
