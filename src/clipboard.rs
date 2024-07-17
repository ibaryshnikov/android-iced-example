use jni::objects::{JObject, JString, JValue};

use crate::java::{get_env, get_vm};

pub(crate) struct Clipboard {}

impl iced_core::Clipboard for Clipboard {
    fn read(&self, _kind: iced_core::clipboard::Kind) -> Option<String> {
        log::debug!("Clipboard read method called");
        match read_clipboard() {
            Ok(text) => Some(text),
            Err(e) => {
                log::error!("Error reading from clipboard: {e}");
                None
            }
        }
    }
    fn write(&mut self, _kind: iced_core::clipboard::Kind, contents: String) {
        log::debug!("Clipboard write method called");
        if let Err(e) = write_clipboard(contents) {
            log::error!("Error writing to clipboard: {e}");
        }
    }
}

fn read_clipboard() -> jni::errors::Result<String> {
    let ctx = ndk_context::android_context();
    let vm = get_vm(&ctx);
    let mut env = get_env(&vm);
    let activity = unsafe { JObject::from_raw(ctx.context() as _) };
    let j_object = env
        .call_method(activity, "readClipboard", "()Ljava/lang/String;", &[])?
        .l()?;
    let j_string = JString::from(j_object);
    env.get_string(&j_string).map(Into::into)
}
fn write_clipboard(contents: String) -> jni::errors::Result<()> {
    let ctx = ndk_context::android_context();
    let vm = get_vm(&ctx);
    let mut env = get_env(&vm);
    let activity = unsafe { JObject::from_raw(ctx.context() as _) };
    let value = env.new_string(contents)?;
    let args = [JValue::Object(value.as_ref())];
    env.call_method(activity, "writeClipboard", "(Ljava/lang/String;)V", &args)?;
    Ok(())
}
