use crate::{
    common::ConnectionEvent,
    connection::ConnectionObserver,
    device::*,
    nal::Nal, trace_err, util::StrResult,
};
//use alvr_common::prelude::*;
use bytes::Bytes;
use jni::{
    JNIEnv, JavaVM, objects::{GlobalRef, JObject, JString, JValue, JValueOwned}
};
use log::debug;
use serde_json;

const INT_TYPE: &'static str = "I";
const FLOAT_TYPE: &'static str = "F";
const BOOLEAN_TYPE: &'static str = "Z";
const FLOAT_ARRAY_TYPE: &'static str = "[F";
const STRING_TYPE: &'static str = "Ljava/lang/String;";

fn get_int_field(env: &mut JNIEnv, object: &JObject, field_name: &str) -> i32 {
    match env.get_field(object, field_name, INT_TYPE).unwrap() {
        JValueOwned::Int(value) => value,
        _ => 0
    }
}

fn get_float_field(env: &mut JNIEnv, object: &JObject, field_name: &str) -> f32 {
    match env.get_field(object, field_name, FLOAT_TYPE).unwrap() {
        JValueOwned::Float(value) => value,
        _ => 0.0
    }
}

fn get_boolean_field(env: &mut JNIEnv, object: &JObject, field_name: &str) -> u8 {
    match env.get_field(object, field_name, BOOLEAN_TYPE).unwrap() {
        JValueOwned::Bool(value) => value,
        _ => 0
    }
}

fn get_string_field(env: &mut JNIEnv, object: &JObject, field_name: &str) -> String {
    match env.get_field(object, field_name, STRING_TYPE).unwrap() {
        JValueOwned::Object(object) => {
            env.get_string(&JString::from(object)).unwrap().into()
        }
        _ => "".into()
    }
}

fn get_float_array_field(env: &JNIEnv, object: &JObject, field_name: &str) -> Vec<f32> {
    match env.get_field(object, field_name, FLOAT_ARRAY_TYPE).unwrap() {
        JValueOwned::Object(object) => {
            let array = jni::objects::JPrimitiveArray::from(object);
            let length = env.get_array_length(&array).unwrap();
            let mut buffer = vec![0.0f32; length as usize];
            env.get_float_array_region(
                array,
                0,
                buffer.as_mut_slice(),
            ).unwrap();
            buffer
        }
        _ => Vec::new()
    }
}

pub struct Preferences<'a> {
    env: JNIEnv<'a>,
    object: JObject<'a>,
}

impl<'a> Preferences<'a> {
    pub fn new(env: JNIEnv<'a>, object: JObject<'a>) -> Self {
        Preferences {
            env,
            object,
        }
    }

    pub fn is_empty(&mut self) -> bool {
        self.get_hostname().is_empty() ||
            self.get_certificate_pem().is_empty() ||
            self.get_key_pem().is_empty()
    }

    pub fn set_hostname(&mut self, value: &str) {
        self.set_string_field("hostname", value)
    }

    pub fn get_hostname(&mut self) -> String {
        get_string_field(&mut self.env, &self.object, "hostname")
    }

    pub fn set_certificate_pem(&mut self, value: &str) {
        self.set_string_field("certificate_pem", value)
    }

    pub fn get_certificate_pem(&mut self) -> String {
        get_string_field(&mut self.env, &self.object, "certificate_pem")
    }

    pub fn set_key_pem(&mut self, value: &str) {
        self.set_string_field("key_pem", value)
    }

    pub fn get_key_pem(&mut self) -> String {
        get_string_field(&mut self.env, &self.object, "key_pem")
    }

    fn set_string_field(&mut self, field_name: &str, value: &str) {
        let j_string = self.env.new_string(value).unwrap();
        self.env.set_field(&self.object, field_name, STRING_TYPE, (&j_string).into()).unwrap()
    }
}

pub struct JDeviceSettings<'a> {
    env: JNIEnv<'a>,
    object: JObject<'a>,
}

impl<'a> JDeviceSettings<'a> {
    pub fn new(env: JNIEnv<'a>, object: JObject<'a>) -> Self {
        JDeviceSettings {
            env,
            object,
        }
    }

    pub fn get_name(&mut self) -> String {
        get_string_field(&mut self.env, &self.object, "name")
    }

    pub fn get_recommended_eye_width(&mut self) -> u32 {
        get_int_field(&mut self.env, &self.object, "recommendedEyeWidth") as u32
    }

    pub fn get_recommended_eye_height(&mut self) -> u32 {
        get_int_field(&mut self.env, &self.object, "recommendedEyeHeight") as u32
    }

    pub fn get_available_refresh_rates(&mut self) -> Vec<f32> {
        get_float_array_field(&mut self.env, &self.object, "availableRefreshRates")
    }

    pub fn get_preferred_refresh_rate(&mut self) -> f32 {
        get_float_field(&mut self.env, &self.object, "preferredRefreshRate")
    }
}

impl From<JDeviceSettings<'_>> for Device {
    fn from(mut settings: JDeviceSettings) -> Self {
        Device {
            name: settings.get_name(),
            recommended_eye_width: settings.get_recommended_eye_width(),
            recommended_eye_height: settings.get_recommended_eye_height(),
            available_refresh_rates: settings.get_available_refresh_rates(),
            preferred_refresh_rate: settings.get_preferred_refresh_rate(),
        }
    }
}

pub struct InputBuffer {
    object: GlobalRef,
}

unsafe impl Sync for InputBuffer {}

unsafe impl Send for InputBuffer {}

impl InputBuffer {
    pub fn new(env: JNIEnv, object: JObject) -> StrResult<Self> {
        Ok(InputBuffer {
            object: trace_err!(env.new_global_ref(object))?,
        })
    }

    pub fn queue_config(&self, env: &mut JNIEnv, nal: Nal) -> StrResult {
        debug!(
            "queue_config {:?} frame_len={} frame_index={}",
            nal.nal_type, nal.frame_buffer.len(), nal.frame_index
        );
        self.copy_buffer(env, &nal.frame_buffer);
        trace_err!(env.call_method(
            &self.object, "queueConfig", "()V", &[]
        ))?;
        Ok(())
    }

    pub fn queue(&self, env: &mut JNIEnv, nal: Nal) -> StrResult {
        debug!(
            "queue {:?} frame_len={} frame_index={}",
            nal.nal_type, nal.frame_buffer.len(), nal.frame_index
        );
        self.copy_buffer(env, &nal.frame_buffer);
        trace_err!(env.call_method(
            &self.object, "queue", "(J)V", &[(nal.frame_index as i64).into()]
        ))?;
        Ok(())
    }

    fn copy_buffer(&self, env: &JNIEnv, frame_buffer: &Bytes) {
        let ret_value = trace_err!(env.call_method(
            &self.object, "getBuffer", "()Ljava/nio/ByteBuffer;", &[]
        )).unwrap();
        if let JValueOwned::Object(byte_buffer) = ret_value {
            let buffer = trace_err!(env.get_direct_buffer_address((&byte_buffer).into())).unwrap();
            buffer[..frame_buffer.len()].copy_from_slice(&frame_buffer);
            trace_err!(env.call_method(
                byte_buffer, "position", "(I)Ljava/nio/Buffer;",
                &[(frame_buffer.len() as i32).into()]
            )).unwrap();
        } else {
            panic!("Can't get the byte buffer.");
        }
    }
}

pub struct JConnectionObserver {
    vm: JavaVM,
    object: GlobalRef,
}

impl JConnectionObserver {
    pub fn new(env: &JNIEnv, object: JObject) -> StrResult<Self> {
        Ok(JConnectionObserver {
            vm: trace_err!(env.get_java_vm())?,
            object: trace_err!(env.new_global_ref(object))?,
        })
    }
}

impl ConnectionObserver for JConnectionObserver {
    fn on_event_occurred(&self, event: ConnectionEvent) -> StrResult {
        let mut env = trace_err!(self.vm.attach_current_thread_permanently())?;
        let json_data = trace_err!(serde_json::to_string(&event))?;
        trace_err!(env.call_method(
            &self.object, "onEventOccurred", "(Ljava/lang/String;)V", &[
                (&trace_err!(env.new_string(json_data))?).into()
            ]
        ))?;
        Ok(())
    }
}

pub struct JDeviceAdapter {
    vm: JavaVM,
    object: GlobalRef,
}

impl JDeviceAdapter {
    pub fn new(env: &JNIEnv, object: JObject) -> StrResult<Self> {
        Ok(JDeviceAdapter {
            vm: trace_err!(env.get_java_vm())?,
            object: trace_err!(env.new_global_ref(object))?,
        })
    }
}

impl DeviceAdapter for JDeviceAdapter {
    fn get_device(&self) -> StrResult<Device> {
        let mut env = trace_err!(self.vm.attach_current_thread_permanently())?;
        let ret = trace_err!(env.call_method(
            &self.object,
            "getDeviceSettings",
            "()Lio/github/alvr/android/lib/DeviceSettings;",
            &[]
        ))?;
        let mut device_settings = JDeviceSettings::new(env, ret.l().unwrap());
        Ok(Device {
            name: device_settings.get_name(),
            recommended_eye_width: device_settings.get_recommended_eye_width(),
            recommended_eye_height: device_settings.get_recommended_eye_height(),
            available_refresh_rates: device_settings.get_available_refresh_rates(),
            preferred_refresh_rate: device_settings.get_preferred_refresh_rate(),
        })
    }

    fn get_tracking(&self, frame_index: u64) -> StrResult<Tracking> {
        let mut env = trace_err!(self.vm.attach_current_thread_permanently())?;
        let ret = trace_err!(env.call_method(
            &self.object,
            "getTracking",
            "(J)Lio/github/alvr/android/lib/Tracking;",
            &[
                (frame_index as i64).into()
            ]
        ))?;
        let mut tracking = JTracking::new(env, ret.l().unwrap());
        let eye_fov = tracking.get_eye_fov();
        let head_pose = tracking.get_head_pose();
        Ok(Tracking {
            ipd: tracking.get_ipd(),
            battery: tracking.get_battery(),
            plugged: tracking.get_plugged(),
            mounted: tracking.get_mounted(),
            l_eye_fov: Rect {
                left: eye_fov[0],
                right: eye_fov[1],
                top: eye_fov[2],
                bottom: eye_fov[3],
            },
            r_eye_fov: Rect {
                left: eye_fov[4],
                right: eye_fov[5],
                top: eye_fov[6],
                bottom: eye_fov[7],
            },
            head_pose_orientation: Quaternion {
                x: head_pose[0],
                y: head_pose[1],
                z: head_pose[2],
                w: head_pose[3],
            },
            head_pose_position: Vector3 {
                x: head_pose[4],
                y: head_pose[5],
                z: head_pose[6],
            },
            l_ctrl: Controller {
                enabled: 0,
                buttons: 0,
                trackpad_position_x: 0.0,
                trackpad_position_y: 0.0,
                trigger_value: 0.0,
                grip_value: 0.0,
                orientation: Quaternion {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 0.0
                },
                position: Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0
                }
            },
            r_ctrl: Controller {
                enabled: 0,
                buttons: 0,
                trackpad_position_x: 0.0,
                trackpad_position_y: 0.0,
                trigger_value: 0.0,
                grip_value: 0.0,
                orientation: Quaternion {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 0.0
                },
                position: Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0
                }
            },
        })
    }

    fn on_rendered(&self, frame_index: u64) -> StrResult<()> {
        let mut env = trace_err!(self.vm.attach_current_thread_permanently())?;
        trace_err!(env.call_method(
            &self.object,
            "onRendered",
            "(J)V",
            &[
                (frame_index as i64).into()
            ]
        ))?;
        Ok(())
    }
}

struct JTracking<'a> {
    env: JNIEnv<'a>,
    object: JObject<'a>,
}

impl<'a> JTracking<'a> {
    pub fn new(env: JNIEnv<'a>, object: JObject<'a>) -> Self {
        JTracking {
            env,
            object,
        }
    }

    pub fn get_ipd(&mut self) -> f32 {
        get_float_field(&mut self.env, &self.object, "ipd")
    }

    pub fn get_battery(&mut self) -> Percentage {
        get_int_field(&mut self.env, &self.object, "battery") as Percentage
    }

    pub fn get_plugged(&mut self) -> u8 {
        get_boolean_field(&mut self.env, &self.object, "plugged")
    }

    pub fn get_mounted(&mut self) -> u8 {
        get_boolean_field(&mut self.env, &self.object, "mounted")
    }

    /// (l.left, l.right, l.top, l.bottom, r.left, r.right, r.top, r.bottom)
    pub fn get_eye_fov(&mut self) -> Vec<f32> {
        get_float_array_field(&mut self.env, &self.object, "eyeFov")
    }

    /// (o.x, o.y, o.z, o.w, p.x, p.y, p.z)
    pub fn get_head_pose(&mut self) -> Vec<f32> {
        get_float_array_field(&mut self.env, &self.object, "headPose")
    }
}