//! GTK presentation only. No host, worker, IPC or hardware access.
use gtk::prelude::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;

pub fn show(
    parent: &impl IsA<gtk::Window>,
    description: &str,
    deadline: Instant,
    pending: Arc<AtomicBool>,
) -> Result<(), String> {
    let parent = parent.as_ref();
    if !parent.is_visible() || !pending.load(Ordering::SeqCst) || Instant::now() >= deadline {
        return Err("NATIVE_CONFIRMATION_UNAVAILABLE".into());
    }
    let previous_focus = parent.focused_widget();
    let dialog = gtk::MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .destroy_with_parent(true)
        .message_type(gtk::MessageType::Warning)
        .text("Autorizar operação local?")
        .secondary_text(description)
        .use_markup(false)
        .secondary_use_markup(false)
        .build();
    dialog.add_button("Cancelar", gtk::ResponseType::Cancel);
    dialog.add_button("Autorizar esta operação", gtk::ResponseType::Accept);
    dialog.set_default_response(gtk::ResponseType::Cancel);
    let first_response = std::rc::Rc::new(std::cell::Cell::new(None));
    let first = first_response.clone();
    dialog.connect_response(move |_, response| {
        if first.get().is_none() {
            first.set(Some(response));
        }
    });
    dialog.show_all();
    if let Some(cancel) = dialog.widget_for_response(gtk::ResponseType::Cancel) {
        cancel.grab_focus();
        cancel.grab_default();
    }

    // Watch both deadlines and worker cancellation. Do not leave a stale dialog
    // live until its original timeout if its worker has already expired.
    let expired = std::rc::Rc::new(std::cell::Cell::new(false));
    let fired = expired.clone();
    let weak = dialog.downgrade();
    let active = pending.clone();
    let timer = gtk::glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        if Instant::now() >= deadline || !active.load(Ordering::SeqCst) {
            fired.set(true);
            if let Some(dialog) = weak.upgrade() {
                dialog.response(gtk::ResponseType::Cancel);
            }
            gtk::glib::ControlFlow::Break
        } else {
            gtk::glib::ControlFlow::Continue
        }
    });
    let returned = dialog.run();
    let response = first_response.get().unwrap_or(returned);
    if !expired.get() {
        timer.remove();
    }
    dialog.close();
    if parent.is_visible() {
        if let Some(widget) = previous_focus {
            if widget.is_visible() && widget.is_sensitive() {
                widget.grab_focus();
            }
        }
    }
    if response == gtk::ResponseType::Accept
        && !expired.get()
        && pending.load(Ordering::SeqCst)
        && Instant::now() < deadline
    {
        Ok(())
    } else {
        Err("NATIVE_CONFIRMATION_DENIED".into())
    }
}
