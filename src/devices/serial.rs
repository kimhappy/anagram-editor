use std::time::Duration;

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    ReadableStreamDefaultReader, Serial, SerialOptions, SerialPort, SerialPortFilter,
    SerialPortRequestOptions,
};

use super::{DeviceError, navigator, navigator_has, with_timeout};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortFilter {
    pub usb_vendor_id: u16,
    pub usb_product_id: Option<u16>,
}

pub mod anagram {
    use super::PortFilter;
    use crate::devices::anagram::{USB_PRODUCT_ID, USB_VENDOR_ID};

    pub const FILTER: PortFilter = PortFilter {
        usb_vendor_id: USB_VENDOR_ID,
        usb_product_id: Some(USB_PRODUCT_ID),
    };
    pub const BAUD_RATE: u32 = 115_200;
}

impl From<PortFilter> for SerialPortFilter {
    fn from(filter: PortFilter) -> Self {
        let serial_filter = Self::new();
        serial_filter.set_usb_vendor_id(filter.usb_vendor_id);
        filter
            .usb_product_id
            .into_iter()
            .for_each(|id| serial_filter.set_usb_product_id(id));
        serial_filter
    }
}

fn serial() -> Result<Serial, DeviceError> {
    navigator_has("serial")
        .then(|| navigator().serial())
        .ok_or(DeviceError::Unsupported("Web Serial"))
}

pub async fn granted_port(filter: PortFilter) -> Result<Option<SerialPort>, DeviceError> {
    let ports = JsFuture::from(serial()?.get_ports()).await?;
    Ok(js_sys::Array::from(&ports)
        .iter()
        .filter_map(|port| port.dyn_into::<SerialPort>().ok())
        .find(|port| {
            let info = port.get_info();
            info.get_usb_vendor_id() == Some(filter.usb_vendor_id)
                && filter
                    .usb_product_id
                    .is_none_or(|id| info.get_usb_product_id() == Some(id))
        }))
}

pub async fn request_port(filters: &[PortFilter]) -> Result<SerialPort, DeviceError> {
    let serial_filters: Vec<SerialPortFilter> = filters.iter().copied().map(Into::into).collect();
    let options = SerialPortRequestOptions::new();
    options.set_filters(&serial_filters);
    Ok(serial()?.request_port_with_options(&options).await?)
}

pub async fn open(port: &SerialPort, baud_rate: u32) -> Result<(), DeviceError> {
    port.open(&SerialOptions::new(baud_rate)).await?;
    Ok(())
}

pub async fn close(port: &SerialPort) -> Result<(), DeviceError> {
    port.close().await?;
    Ok(())
}

pub async fn write(port: &SerialPort, data: &[u8]) -> Result<(), DeviceError> {
    let writer = port.writable().get_writer()?;
    let outcome = JsFuture::from(writer.write_with_chunk(&js_sys::Uint8Array::from(data))).await;
    writer.release_lock();
    outcome.map(|_| ()).map_err(DeviceError::from)
}

pub async fn read_exact(
    port: &SerialPort,
    length: usize,
    timeout: Duration,
) -> Result<Vec<u8>, DeviceError> {
    let reader: ReadableStreamDefaultReader = port.readable().get_reader().unchecked_into();
    let mut bytes = Vec::with_capacity(length);
    let outcome = with_timeout(read_into(&reader, &mut bytes, length), timeout).await;
    if outcome.is_none() {
        JsFuture::from(reader.cancel()).await.unwrap_or_default();
    }
    reader.release_lock();
    outcome.ok_or_else(|| {
        DeviceError::Browser("The device did not answer over the serial port.".to_owned())
    })??;
    if bytes.len() < length {
        return Err(DeviceError::Browser(
            "The serial port closed before the device answered.".to_owned(),
        ));
    }
    Ok(bytes)
}

async fn read_into(
    reader: &ReadableStreamDefaultReader,
    bytes: &mut Vec<u8>,
    length: usize,
) -> Result<(), DeviceError> {
    while bytes.len() < length {
        let result = JsFuture::from(reader.read()).await?;
        let done = js_sys::Reflect::get(&result, &JsValue::from_str("done"))?
            .as_bool()
            .unwrap_or(true);
        let value = js_sys::Reflect::get(&result, &JsValue::from_str("value"))?;
        if let Some(chunk) = value.dyn_ref::<js_sys::Uint8Array>() {
            bytes.extend(chunk.to_vec());
        }
        if done {
            break;
        }
    }
    Ok(())
}
