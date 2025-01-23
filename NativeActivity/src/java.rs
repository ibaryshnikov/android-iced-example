use jni::objects::JObject;
use jni::{AttachGuard, JavaVM};

//
// some jni syntax hints
//
// L - class, for example: Ljava/lang/String;
// primitives: Z - boolean, I - integer, V - void
// for example, Rust signature fn get_text(flag: bool) -> String
// will become "(Z)Ljava/lang/String;"
//
// in find_class . is replaced by $
// docs: android/view/WindowManager.LayoutParams
// jni:  android/view/WindowManager$LayoutParams
// it also has some quirks:
// https://developer.android.com/training/articles/perf-jni.html#faq_FindClass
//

pub(crate) fn call_instance_method(name: &str) {
    log::debug!("Calling instance method from Rust: {}", name);
    let ctx = ndk_context::android_context();
    let vm = get_vm(&ctx);
    let mut env = get_env(&vm);
    let activity = unsafe { JObject::from_raw(ctx.context() as _) };
    if let Err(e) = env.call_method(activity, name, "()V", &[]) {
        log::error!("Error calling instance method {}: {}", name, e);
    }
}

pub(crate) fn get_vm(ctx: &ndk_context::AndroidContext) -> JavaVM {
    unsafe { JavaVM::from_raw(ctx.vm() as _) }.unwrap_or_else(|e| {
        log::error!("Error getting ctx.vm(): {:?}", e);
        panic!("No JavaVM found");
    })
}

pub(crate) fn get_env(vm: &JavaVM) -> AttachGuard {
    vm.attach_current_thread().unwrap_or_else(|e| {
        log::error!("Error attaching vm: {:?}", e);
        panic!("Failed to call attach_current_thread for JavaVM");
    })
}
