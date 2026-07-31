use eframe::{egui, Frame, App};
use eframe::egui::Ui;
use egui::ScrollArea;
use crate::{set_port_range, set_target_ip, set_working_mode};
use crate::subfiles::mt::smart_scanning;
use crate::subfiles::mt::{reset_results, ResultType};
use crate::{ScanResult};
use std::collections::BTreeSet;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum OperatingMode {
    Safe,
    Fast,
}

pub struct Gui {
    ip1: u16,
    ip2: u16,
    ip3: u16,
    ip4: u16,
    port_start: u16,
    port_end: u16,
    operating_mode: OperatingMode,
    pub results: Arc<Mutex<BTreeSet<ScanResult>>>,
    popup_message: Option<String>,
    scanning: Arc<Mutex<bool>>,
    scan_complete_rx: Option<mpsc::Receiver<()>>,
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            ip1: 192,
            ip2: 168,
            ip3: 2,
            ip4: 136,
            port_start: 1,
            port_end: 100,
            operating_mode: OperatingMode::Fast,
            results: Arc::new(Mutex::new(BTreeSet::new())),
            popup_message: None,
            scanning: Arc::new(Mutex::new(false)),
            scan_complete_rx: None,
        }
    }
}

impl Clone for Gui {
    fn clone(&self) -> Self {
        Self {
            ip1: self.ip1,
            ip2: self.ip2,
            ip3: self.ip3,
            ip4: self.ip4,
            port_start: self.port_start,
            port_end: self.port_end,
            operating_mode: self.operating_mode,
            results: Arc::clone(&self.results),
            popup_message: self.popup_message.clone(),
            scanning: Arc::clone(&self.scanning),
            scan_complete_rx: None,
        }
    }
}

impl Gui {

    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    pub fn show_popup(&mut self, message: String) {
        self.popup_message = Some(message);
    }

    fn show_popup_window(&mut self, ctx: &egui::Context) {
        if let Some(message) = self.popup_message.clone() {
            egui::Window::new("Message")
                .collapsible(false)
                .resizable(false)
                .fixed_pos(egui::pos2(100.0, 100.0))
                .show(ctx, |ui| {
                    ui.label(&message);
                    if ui.button("OK").clicked() {
                        self.popup_message = None;
                    }
                });
        }
    }

    pub fn start_scan(ip1: u16, ip2: u16, ip3: u16, ip4: u16, start_port: u16, end_port: u16, mode: OperatingMode, tx: mpsc::SyncSender<ScanResult>) {
        reset_results();
        let ip_address = format!("{}.{}.{}.{}", ip1, ip2, ip3, ip4);
        set_working_mode(&mode);
        set_target_ip(ip_address);
        set_port_range((start_port.to_string(), end_port.to_string()));

        let _ = smart_scanning(Some(tx));

        // if get_results().contains(&ScanResult{port: 22, result: ResultType::Open }) {
        //     check_os_p22()
        // }

    }

    fn show_results(&mut self, ui: &mut Ui) {
        let min_height = 200.0;
        ui.add_space(10.0);
        ui.heading("Scan Results");
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            let results = self.results.try_lock().unwrap();

            // Open Ports
            ui.vertical(|ui| {
                ui.set_max_width(220.0);
                ui.heading("Open Ports");
                ui.add_space(5.0);
                ScrollArea::vertical()
                    .id_salt("open_ports")
                    .auto_shrink([false; 2])
                    .min_scrolled_height(min_height)
                    .show(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        for line in results.iter().filter(|r| r.result == ResultType::Open) {
                            ui.label(line.to_string());
                        }
                    });
            });

            ui.add_space(10.0);

            // Restricted/Filtered Ports
            ui.vertical(|ui| {
                ui.set_max_width(220.0);
                ui.heading("Restricted Ports");
                ui.add_space(5.0);
                ScrollArea::vertical()
                    .id_salt("restricted_ports")
                    .auto_shrink([false; 2])
                    .min_scrolled_height(min_height)
                    .show(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        for line in results.iter().filter(|r| r.result == ResultType::Filtered) {
                            ui.label(line.to_string());
                        }
                    });
            });

            ui.add_space(10.0);

            // Closed Ports
            ui.vertical(|ui| {
                ui.set_max_width(220.0);
                ui.heading("Closed Ports");
                ui.add_space(5.0);
                ScrollArea::vertical()
                    .id_salt("closed_ports")
                    .auto_shrink([false; 2])
                    .min_scrolled_height(min_height)
                    .show(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        for line in results.iter().filter(|r| r.result == ResultType::Closed) {
                            ui.label(line.to_string());
                        }
                    });
            });

            ui.add_space(10.0);

            // Error Ports
            ui.vertical(|ui| {
                ui.set_max_width(220.0);
                ui.heading("Error Ports");
                ui.add_space(5.0);
                ScrollArea::vertical()
                    .id_salt("error_ports")
                    .auto_shrink([false; 2])
                    .min_scrolled_height(min_height)
                    .show(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        for line in results.iter().filter(|r| r.result == ResultType::Error) {
                            ui.label(line.to_string());
                        }
                    });
            });
        });
    }
}



// TODO: Separate Listen für Ergebnisse
impl App for Gui {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        // Check for scan completion
        if let Some(ref mut rx) = self.scan_complete_rx {
            if let Ok(()) = rx.try_recv() {
                self.show_popup("Scan completed!".to_string());
                self.scan_complete_rx = None;
            }
        }

        // Request repaint while scanning to keep UI updated
        if *self.scanning.lock().unwrap() {
            ui.ctx().request_repaint();
        }

        self.show_popup_window(ui.ctx());

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Choose operating mode");
            ui.horizontal(|ui| {
                if ui.add_enabled(!*self.scanning.lock().unwrap(), egui::RadioButton::new(self.operating_mode == OperatingMode::Safe, "Safe")).clicked() {
                    self.operating_mode = OperatingMode::Safe;
                }
                if ui.add_enabled(!*self.scanning.lock().unwrap(), egui::RadioButton::new(self.operating_mode == OperatingMode::Fast, "Fast")).clicked() {
                    self.operating_mode = OperatingMode::Fast;
                }
            });

            ui.add_space(10.0);

            ui.heading("Choose IP");
            ui.horizontal(|ui| {
                ui.add_enabled(!*self.scanning.lock().unwrap(), egui::DragValue::new(&mut self.ip1));
                ui.label(".");
                ui.add_enabled(!*self.scanning.lock().unwrap(), egui::DragValue::new(&mut self.ip2));
                ui.label(".");
                ui.add_enabled(!*self.scanning.lock().unwrap(), egui::DragValue::new(&mut self.ip3));
                ui.label(".");
                ui.add_enabled(!*self.scanning.lock().unwrap(), egui::DragValue::new(&mut self.ip4));
            });

            ui.add_space(10.0);

            ui.heading("Choose target port range");
            ui.horizontal(|ui| {
                ui.add_enabled(!*self.scanning.lock().unwrap(), egui::DragValue::new(&mut self.port_start));
                ui.label("-");
                ui.add_enabled(!*self.scanning.lock().unwrap(), egui::DragValue::new(&mut self.port_end));
            });

            ui.add_space(10.0);

            // Beim Klick rufen wir die Start-Methode auf
            if ui.add_enabled(!*self.scanning.lock().unwrap(), egui::Button::new("Start Scan")).clicked() {
                println!("Start scan");
                self.results.lock().unwrap().clear();
                *self.scanning.lock().unwrap() = true;

                // Use a bounded channel with capacity 100 to prevent backlog
                let (tx, rx) = mpsc::sync_channel::<ScanResult>(100);

                let gui_copy = self.clone();
                let scanning_clone = Arc::clone(&self.scanning);
                let results_clone = Arc::clone(&self.results);

                // Use a separate channel for completion notification
                let (complete_tx, complete_rx) = mpsc::channel();

                thread::spawn(move || {
                    println!("Scanning...");
                    Gui::start_scan(gui_copy.ip1, gui_copy.ip2, gui_copy.ip3, gui_copy.ip4, gui_copy.port_start, gui_copy.port_end, gui_copy.operating_mode, tx);
                    *scanning_clone.lock().unwrap() = false;
                });

                // Receiver thread to process results and send completion when done
                thread::spawn(move || {
                    while let Ok(result) = rx.recv() {
                        results_clone.lock().unwrap().insert(result);
                    }
                    // Channel is closed and all results processed - send completion
                    let _ = complete_tx.send(());
                });

                self.scan_complete_rx = Some(complete_rx);
            }

            ui.add_space(10.0);


            self.show_results(ui);
        });
    }
}
