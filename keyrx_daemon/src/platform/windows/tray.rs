use crossbeam_channel::Receiver;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

#[allow(dead_code)]
fn load_icon(bytes: &[u8]) -> Result<Icon, Box<dyn std::error::Error>> {
    let image = image::load_from_memory(bytes)?.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Icon::from_rgba(rgba, width, height).map_err(|e| e.into())
}

#[allow(dead_code)]
pub enum TrayControlEvent {
    Reload,
    Exit,
}

#[allow(dead_code)]
pub struct TrayIconController {
    _tray_icon: TrayIcon,
    menu_receiver: Receiver<MenuEvent>,
    reload_id: String,
    exit_id: String,
}

impl TrayIconController {
    #[allow(dead_code)]
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tray_menu = Menu::new();
        let reload_item = MenuItem::new("Reload Config", true, None);
        let exit_item = MenuItem::new("Exit", true, None);

        tray_menu.append_items(&[&reload_item, &PredefinedMenuItem::separator(), &exit_item])?;

        let reload_id = reload_item.id().0.clone();
        let exit_id = exit_item.id().0.clone();

        let icon_bytes = include_bytes!("../../../assets/icon.png");
        let icon = load_icon(icon_bytes)?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("KeyRx Daemon")
            .with_icon(icon)
            .build()?;

        let menu_receiver = MenuEvent::receiver().clone();

        Ok(Self {
            _tray_icon: tray_icon,
            menu_receiver,
            reload_id,
            exit_id,
        })
    }

    #[allow(dead_code)]
    pub fn poll_event(&self) -> Option<TrayControlEvent> {
        if let Ok(event) = self.menu_receiver.try_recv() {
            if event.id.0 == self.reload_id {
                return Some(TrayControlEvent::Reload);
            } else if event.id.0 == self.exit_id {
                return Some(TrayControlEvent::Exit);
            }
        }
        None
    }
}
