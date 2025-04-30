use accessibility::{AXUIElement, ElementFinder};
use accessibility_sys::{kAXErrorSuccess, kAXPositionAttribute, kAXSizeAttribute, kAXTitleAttribute, kAXValueTypeCGPoint, AXUIElementCopyAttributeValue, AXUIElementCreateApplication, AXValueGetValue};
use accessibility_sys::{AXIsProcessTrustedWithOptions, kAXTrustedCheckOptionPrompt};
use core_foundation::base::*;
use core_foundation::number::*;
use core_foundation::string::*;
use core_foundation_sys::base::{CFRelease, TCFTypeRef, CFGetTypeID};
use core_foundation_sys::dictionary::{CFDictionaryAddValue, CFDictionaryCreateMutable, CFDictionaryGetCount, CFDictionaryGetKeysAndValues};
use core_foundation_sys::number::{kCFBooleanFalse, kCFBooleanTrue};
use core_graphics::display::*;
use core_graphics2::geometry::CGRectMakeWithDictionaryRepresentation;
use std::ffi::{CStr, c_void};
use std::{error::Error, ptr};

fn check_accessibility(ask_if_not_allowed: bool) -> Result<bool, Box<dyn Error>> {
    let is_allowed;
    unsafe {
        let options =
            CFDictionaryCreateMutable(ptr::null_mut(), 0, std::ptr::null(), std::ptr::null());
        let key = kAXTrustedCheckOptionPrompt;
        let value = if ask_if_not_allowed {
            kCFBooleanTrue
        } else {
            kCFBooleanFalse
        };
        if !options.is_null() {
            CFDictionaryAddValue(options, key.as_void_ptr(), value.as_void_ptr());
            is_allowed = AXIsProcessTrustedWithOptions(options);
            CFRelease(options as *const _);
        } else {
            return Err("options is null".into());
        }
    }
    Ok(is_allowed)
}

fn walk() {
    let sys_wide = AXUIElement::system_wide();

    let finder = ElementFinder::new(&sys_wide, |_| true, None);

    let el = finder.find().unwrap();

    println!("{:?}", el.attribute_names())
}

fn window_owners() {
    // const OPTIONS: CGWindowListOption =
    //     kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements;
    const OPTIONS: CGWindowListOption =
        kCGWindowListOptionAll;
    let window_list_info: *const core_foundation::array::__CFArray = unsafe { CGWindowListCopyWindowInfo(OPTIONS, kCGNullWindowID) };
    let count = unsafe { CFArrayGetCount(window_list_info) };

    for i in 0..count {
        let dic_ref =
            unsafe { CFArrayGetValueAtIndex(window_list_info, i as isize) as CFDictionaryRef };

        print_dict(dic_ref);

        let key = CFString::new("kCGWindowOwnerName");
        let mut value: *const c_void = std::ptr::null();

        if unsafe { CFDictionaryGetValueIfPresent(dic_ref, key.to_void(), &mut value) != 0 } {
            let cf_ref = value as CFStringRef;
            let c_ptr = unsafe { CFStringGetCStringPtr(cf_ref, kCFStringEncodingUTF8) };
            if !c_ptr.is_null() {
                let c_result = unsafe { CStr::from_ptr(c_ptr) };
                let result = String::from(c_result.to_str().unwrap());
                println!("Window owner name: {}", result)
            }
        }

        

        let key = CFString::new("kCGWindowName");
        let mut value: *const c_void = std::ptr::null();

        if unsafe { CFDictionaryGetValueIfPresent(dic_ref, key.to_void(), &mut value) != 0 } {
            let cf_ref = value as CFStringRef;
            let c_ptr = unsafe { CFStringGetCStringPtr(cf_ref, kCFStringEncodingUTF8) };
            if !c_ptr.is_null() {
                let c_result = unsafe { CStr::from_ptr(c_ptr) };
                let result = String::from(c_result.to_str().unwrap());
                println!("Window name: {}", result)
            }
        }

        // Get window bounds
        let key = CFString::new("kCGWindowBounds");
        let mut value: *const c_void = std::ptr::null();

        if unsafe { CFDictionaryGetValueIfPresent(dic_ref, key.to_void(), &mut value) != 0 } {
            let cf_ref = value as CFDictionaryRef;
            let mut rect = core_graphics2::geometry::CGRect::default();
            if unsafe { CGRectMakeWithDictionaryRepresentation(cf_ref, &mut rect as *mut _) } {
                println!("Window bounds: x={}, y={}, width={}, height={}", 
                    rect.origin.x, rect.origin.y, rect.size.width, rect.size.height);
            }
        }

        let key = CFString::new("kCGWindowOwnerPID");
        let mut value: *const c_void = std::ptr::null();

        if unsafe { CFDictionaryGetValueIfPresent(dic_ref, key.to_void(), &mut value) != 0 } {
            let cf_ref = value as CFNumberRef;
            let mut window_id: i32 = 0;
            if unsafe { CFNumberGetValue(cf_ref, kCFNumberSInt32Type, &mut window_id as *mut _ as *mut c_void) } {
                println!("kCGWindowOwnerPID: {}", window_id);

                unsafe {
                    let app = AXUIElementCreateApplication(window_id);
                    println!("Created AXUIElement for PID: {}", window_id);
                    
                    // Try position
                    let mut pos_value: *const c_void = std::ptr::null();
                    let pos_attr = CFString::new(kAXPositionAttribute);
                    let pos_result = AXUIElementCopyAttributeValue(app, pos_attr.as_concrete_TypeRef(), &mut pos_value);
                    println!("Position attribute result: {}", pos_result);
                    
                    // Try size
                    let mut size_value: *const c_void = std::ptr::null();
                    let size_attr = CFString::new(kAXSizeAttribute);
                    let size_result = AXUIElementCopyAttributeValue(app, size_attr.as_concrete_TypeRef(), &mut size_value);
                    println!("Size attribute result: {}", size_result);
                    
                    // Try title
                    let mut title_value: *const c_void = std::ptr::null();
                    let title_attr = CFString::new(kAXTitleAttribute);
                    let title_result = AXUIElementCopyAttributeValue(app, title_attr.as_concrete_TypeRef(), &mut title_value);
                    println!("Title attribute result: {}", title_result);
                    
                    if pos_result == kAXErrorSuccess {
                        println!("Got position value: {:?}", pos_value);
                        let mut point = core_graphics::geometry::CGPoint { x: 0.0, y: 0.0 };
                        let get_value_result = AXValueGetValue(pos_value as *mut _, kAXValueTypeCGPoint, &mut point as *mut _ as *mut c_void);
                        println!("AXValueGetValue result: {}", get_value_result);
                        if get_value_result {
                            println!("Position: ({}, {})", point.x, point.y);
                        }
                    }
                    
                    if title_result == kAXErrorSuccess {
                        let title_str = title_value as CFStringRef;
                        let title_ptr = CFStringGetCStringPtr(title_str, kCFStringEncodingUTF8);
                        if !title_ptr.is_null() {
                            let title = CStr::from_ptr(title_ptr).to_str().unwrap_or("unknown");
                            println!("Title: {}", title);
                        }
                    }
                    
                    CFRelease(app as CFTypeRef);
                }
            }
        }

        println!("---")
    }

    unsafe { CFRelease(window_list_info as CFTypeRef) }
}

fn print_dict(dict: CFDictionaryRef) {
    unsafe {
        let count = CFDictionaryGetCount(dict);
        let mut keys: Vec<*const c_void> = vec![ptr::null_mut(); count as usize];
        let mut values: Vec<*const c_void> = vec![ptr::null_mut(); count as usize];
        
        CFDictionaryGetKeysAndValues(
            dict,
            keys.as_mut_ptr(),
            values.as_mut_ptr(),
        );

        for i in 0..count {
            let key = keys[i as usize] as CFStringRef;
            let value = values[i as usize];
            
            let key_ptr = CFStringGetCStringPtr(key, kCFStringEncodingUTF8);
            if !key_ptr.is_null() {
                let key_str = CStr::from_ptr(key_ptr).to_str().unwrap_or("unknown");
                print!("{}: ", key_str);
            }

            if CFGetTypeID(value) == CFStringGetTypeID() {
                let value_str = value as CFStringRef;
                let value_ptr = CFStringGetCStringPtr(value_str, kCFStringEncodingUTF8);
                if !value_ptr.is_null() {
                    println!("{}", CStr::from_ptr(value_ptr).to_str().unwrap_or("unknown"));
                }
            } else if CFGetTypeID(value) == CFBooleanGetTypeID() {
                let value_bool = value as CFBooleanRef;
                println!("{}", CFBooleanGetValue(value_bool));
            } else if CFGetTypeID(value) == CFNumberGetTypeID() {
                let value_num = value as CFNumberRef;
                let mut num_value: i32 = 0;
                if CFNumberGetValue(value_num, kCFNumberSInt32Type, &mut num_value as *mut _ as *mut c_void) {
                    println!("{}", num_value);
                }
            } else {
                println!("<unknown type>");
            }
        }
    }
}

fn main() {
    window_owners();
}
