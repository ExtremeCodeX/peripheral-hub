//! Standalone GTK check: deliberately does not import the host or any adapter.
#[cfg(target_os = "linux")]
#[path = "../native_dialog.rs"]
mod native_dialog;

#[cfg(target_os = "linux")]
fn main() -> Result<(), String> {
    use gtk::prelude::*;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::time::{Duration, Instant};
    let automatic = match std::env::args().nth(1).as_deref() {
        Some("--self-test") => true,
        Some("--interactive") => false,
        _ => return Err("Use --self-test or --interactive; this binary has no HID host".into()),
    };
    gtk::init().map_err(|_| "GTK display unavailable")?;
    let parent = gtk::Window::new(gtk::WindowType::Toplevel);
    parent.set_title("Peripheral Hub — teste isolado sem HID");
    let entry = gtk::Entry::new();
    entry.set_text("Foco antes e depois do diálogo");
    parent.add(&entry);
    parent.show_all();
    entry.grab_focus();
    let cases = if automatic { 11 } else { 1 };
    for case in 0..cases {
        let pending = Arc::new(AtomicBool::new(true));
        let checked = std::rc::Rc::new(std::cell::Cell::new(false));
        if automatic {
            let parent = parent.clone();
            let active = pending.clone();
            let checked = checked.clone();
            gtk::glib::timeout_add_local_once(Duration::from_millis(80), move || {
                let dialog = gtk::Window::list_toplevels()
                    .into_iter()
                    .filter(|window| window.is_visible())
                    .find_map(|w| w.downcast::<gtk::MessageDialog>().ok());
                let Some(dialog) = dialog else {
                    return;
                };
                if let Some(cancel) = dialog.widget_for_response(gtk::ResponseType::Cancel) {
                    // Bare Xvfb has no WM/active-window focus; test GTK's logical
                    // focus rather than claim desktop keyboard activation.
                    checked.set(
                        dialog.focused_widget() == Some(cancel.clone())
                            && cancel.has_default()
                            && dialog.is_modal()
                            && dialog.transient_for() == Some(parent.clone()),
                    );
                }
                let checkbox = dialog
                    .content_area()
                    .children()
                    .into_iter()
                    .find_map(|widget| widget.downcast::<gtk::CheckButton>().ok());
                if let Some(checkbox) = &checkbox {
                    assert!(!checkbox.is_active(), "Remember must default to unchecked");
                    if case >= 8 {
                        checkbox.set_active(true);
                    }
                }
                if case >= 7 {
                    assert!(checkbox.is_some());
                }
                match case {
                    0 => dialog.response(gtk::ResponseType::Accept),
                    1 => dialog.response(gtk::ResponseType::Cancel),
                    2 => dialog.response(gtk::ResponseType::DeleteEvent),
                    3 => (), // actual GTK deadline timer
                    4 => active.store(false, Ordering::SeqCst),
                    5 => {
                        dialog.response(gtk::ResponseType::Cancel);
                        dialog.response(gtk::ResponseType::Accept);
                    }
                    10 => unsafe {
                        parent.destroy();
                    },
                    6 | 7 | 8 => dialog.response(gtk::ResponseType::Accept),
                    9 => dialog.response(gtk::ResponseType::Cancel),
                    _ => unreachable!(),
                }
            });
        }
        let result = native_dialog::show(&parent,
            "TESTE ISOLADO — nenhum dispositivo será acessado.\nEsta decisão não concede autorização ao Hub.",
            Instant::now() + Duration::from_millis(if automatic { if case == 3 { 180 } else { 2000 } } else { 120000 }), pending, case >= 7 || !automatic);
        if automatic {
            assert!(
                checked.get(),
                "GTK logical Cancel focus/default/modal/parent check failed for case {case}"
            );
            if matches!(case, 0 | 6 | 7 | 8) {
                assert_eq!(result, Ok(case == 8));
            } else {
                assert!(
                    result.is_err(),
                    "only an explicit first acceptance may succeed"
                );
            }
            if case != 10 {
                assert_eq!(parent.focused_widget(), Some(entry.clone().upcast()));
            }
            println!("GTK case {case}: passed");
        } else {
            println!("Isolated decision: {result:?}; no host or HID exists here");
        }
    }
    unsafe {
        parent.destroy();
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("GTK check requires Linux");
}
