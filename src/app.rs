use crate::config::{self, PortableConfig, ReforgeProfile};
use crate::core::catalog::{catalog, preset_app_ids, AppCategory, CatalogApp, PresetKind};
use crate::core::cleaner::{
    self, CleanCategoryId, CleanExecResult, CleanScanResult,
};
use crate::core::debloater::{self, BloatPackage, BloatSafety};
use crate::core::installer::{
    self, InstallEvent, InstallJob, InstallStage, WingetSearchResult,
};
use crate::core::startup::{self, StartupItem};
use crate::core::tweaks::{self, SystemTweak, TweakCategory, TweakId};
use crate::core::uninstaller::{self, AppTrace, InstalledApp, TraceConfidence, TraceDeleteReport};
use crate::core::updater::{self, AppUpdate, UpdateItemStatus, UpdateProgressEvent};
use crate::core::winapi::{self, SystemInfo};
use crate::i18n::Lang;
use crate::ui::icons;
use crate::ui::theme::{self, ThemeMode, FONT_MEDIUM};
use crate::ui::views;
use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, stack, text, Space};
use iced::{Element, Length, Padding, Subscription, Task, Theme};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Installer,
    Updater,
    Uninstaller,
    Cleaner,
    Debloater,
    Startup,
    Tweaks,
    Settings,
}

#[derive(Debug, Clone)]
pub struct InstallTaskState {
    pub app_id: String,
    pub app_name: String,
    pub stage: InstallStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Navigation & Global
    SelectTab(Tab),
    CycleTheme,
    SetTheme(ThemeMode),
    ToggleLanguage,
    SetLanguage(Lang),
    ElevateAsAdmin,
    SyncDwmTitlebar,
    DismissToast,
    OpenPath(String),
    OpenUrl(String),
    PollStreams,

    // System
    RefreshSystemInfo,
    SystemInfoLoaded(SystemInfo),
    CreateRestorePoint,
    RestorePointFinished(Result<String, String>),
    BootstrapWinget,
    BootstrapWingetFinished(Result<String, String>),

    // Installer
    InstallerSearchChanged(String),
    InstallerCategoryChanged(AppCategory),
    ToggleCatalogApp(&'static str),
    ApplyInstallPreset(PresetKind),
    ClearInstallSelection,
    SearchWingetCatalog,
    WingetSearchFinished(Result<Vec<WingetSearchResult>, String>),
    ClearWingetResults,
    ToggleWingetApp(String, String),
    StartInstallBatch,
    InstallBatchDone,
    ClearInstallQueue,

    // Uninstaller
    RefreshInstalledApps,
    InstalledAppsLoaded(Vec<InstalledApp>),
    UninstallerSearchChanged(String),
    ToggleSystemApps(bool),
    UninstallApp(InstalledApp),
    UninstallFinished(InstalledApp, Result<(), String>),
    ForceRemoveApp(InstalledApp),
    DeepScanApp(InstalledApp),
    DeepScanFinished(Vec<AppTrace>),
    ToggleTrace(usize),
    ToggleAllTraces(bool),
    SelectHighConfidenceTraces,
    DeleteSelectedTraces,
    TracesDeleted(TraceDeleteReport),
    CloseTraceReview,

    // Updater
    CheckUpdates,
    UpdatesChecked(Result<Vec<AppUpdate>, String>),
    ToggleUpdateSelection(String),
    RunSelectedUpdates,
    RunSingleUpdate(String),
    UpdatesBatchDone,

    // Cleaner
    ScanJunk,
    JunkScanned(CleanScanResult),
    ToggleCleanCategory(CleanCategoryId),
    CleanSelectedJunk,
    JunkCleaned(CleanExecResult),

    // Debloater
    ScanBloatware,
    BloatwareScanned(Result<Vec<BloatPackage>, String>),
    ToggleBloatPackage(String),
    SelectSafeBloatOnly,
    RemoveSelectedBloat,
    BloatRemoved(usize, usize),

    // Startup
    RefreshStartupItems,
    StartupItemsLoaded(Vec<StartupItem>),
    StartupSearchChanged(String),
    ToggleStartupItem(String, bool),
    DeleteStartupItem(String),

    // Tweaks
    TweakFilterChanged(Option<TweakCategory>),
    ToggleTweak(TweakId, bool),
    ApplyRecommendedTweaks,
    RestoreAllTweaks,
    TweaksReloaded(Vec<SystemTweak>),

    // Settings / Portable
    SetMaxConcurrency(usize),
    SetAutoCleanup(bool),
    SetPreferQuietUninstall(bool),
    ExportProfile,
    ImportProfile,
}

pub struct App {
    pub active_tab: Tab,
    pub config: PortableConfig,
    pub sys_info: SystemInfo,
    pub toast: Option<Toast>,
    pub winget_bootstrapping: bool,

    // Installer
    pub installer_search: String,
    pub installer_category: AppCategory,
    pub selected_catalog_ids: HashSet<&'static str>,
    pub selected_winget_ids: HashMap<String, String>,
    pub winget_results: Vec<WingetSearchResult>,
    pub winget_searching: bool,
    pub install_queue: Vec<InstallTaskState>,
    pub installing_active: bool,
    install_rx: Option<Arc<Mutex<mpsc::UnboundedReceiver<InstallEvent>>>>,

    // Uninstaller
    pub installed_apps: Vec<InstalledApp>,
    pub uninstaller_search: String,
    pub show_system_apps: bool,
    pub uninstalling_id: Option<String>,
    pub trace_target_app: Option<InstalledApp>,
    pub detected_traces: Vec<AppTrace>,
    pub traces_scanning: bool,

    // Updater
    pub updates: Vec<AppUpdate>,
    pub updates_checking: bool,
    pub updates_checked_once: bool,
    pub updates_running: bool,
    update_rx: Option<Arc<Mutex<mpsc::UnboundedReceiver<UpdateProgressEvent>>>>,

    // Cleaner
    pub clean_scan: CleanScanResult,
    pub cleaner_scanning: bool,
    pub cleaner_cleaning: bool,
    pub last_clean_exec: Option<CleanExecResult>,

    // Debloater
    pub bloat_packages: Vec<BloatPackage>,
    pub bloat_scanning: bool,
    pub bloat_loaded_once: bool,
    pub bloat_removing: bool,

    // Startup
    pub startup_items: Vec<StartupItem>,
    pub startup_search: String,

    // Tweaks
    pub tweaks: Vec<SystemTweak>,
    pub tweak_filter: Option<TweakCategory>,
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        let mut cfg = config::load_config();
        let mut initial_tab = Tab::Installer;

        // Optional lightweight CLI flags (e.g., --tab=cleaner --theme=light --lang=pt)
        for arg in std::env::args().skip(1) {
            if let Some(t) = arg.strip_prefix("--tab=") {
                initial_tab = match t.to_lowercase().as_str() {
                    "installer" => Tab::Installer,
                    "updater" => Tab::Updater,
                    "uninstaller" => Tab::Uninstaller,
                    "cleaner" => Tab::Cleaner,
                    "debloater" => Tab::Debloater,
                    "startup" => Tab::Startup,
                    "tweaks" => Tab::Tweaks,
                    "settings" => Tab::Settings,
                    _ => Tab::Installer,
                };
            } else if let Some(th) = arg.strip_prefix("--theme=") {
                cfg.theme = match th.to_lowercase().as_str() {
                    "dark" => ThemeMode::Dark,
                    "light" => ThemeMode::Light,
                    _ => ThemeMode::System,
                };
            } else if let Some(l) = arg.strip_prefix("--lang=") {
                cfg.lang = match l.to_lowercase().as_str() {
                    "pt" | "pt-br" => Lang::PtBr,
                    _ => Lang::En,
                };
            }
        }

        let app = Self {
            active_tab: initial_tab,
            config: cfg,
            sys_info: SystemInfo::default(),
            toast: None,
            winget_bootstrapping: false,

            installer_search: String::new(),
            installer_category: AppCategory::All,
            selected_catalog_ids: HashSet::new(),
            selected_winget_ids: HashMap::new(),
            winget_results: Vec::new(),
            winget_searching: false,
            install_queue: Vec::new(),
            installing_active: false,
            install_rx: None,

            installed_apps: Vec::new(),
            uninstaller_search: String::new(),
            show_system_apps: false,
            uninstalling_id: None,
            trace_target_app: None,
            detected_traces: Vec::new(),
            traces_scanning: false,

            updates: Vec::new(),
            updates_checking: false,
            updates_checked_once: false,
            updates_running: false,
            update_rx: None,

            clean_scan: cleaner::initial_categories(),
            cleaner_scanning: true,
            cleaner_cleaning: false,
            last_clean_exec: None,

            bloat_packages: Vec::new(),
            bloat_scanning: false,
            bloat_loaded_once: false,
            bloat_removing: false,

            startup_items: Vec::new(),
            startup_search: String::new(),

            tweaks: Vec::new(),
            tweak_filter: None,
        };

        // Fire all fast native scans in parallel on boot so every tab feels instantaneous!
        let mut tasks = vec![
            Task::perform(
                tokio::task::spawn_blocking(winapi::collect_system_info),
                |r| Message::SystemInfoLoaded(r.unwrap_or_default()),
            ),
            Task::perform(
                tokio::task::spawn_blocking(|| uninstaller::scan_installed_apps(false)),
                |r| Message::InstalledAppsLoaded(r.unwrap_or_default()),
            ),
            Task::perform(
                tokio::task::spawn_blocking(|| cleaner::scan_system_junk(None)),
                |r| Message::JunkScanned(r.unwrap_or_default()),
            ),
            Task::perform(
                tokio::task::spawn_blocking(startup::scan_startup_items),
                |r| Message::StartupItemsLoaded(r.unwrap_or_default()),
            ),
            Task::perform(
                tokio::task::spawn_blocking(tweaks::load_all_tweaks),
                |r| Message::TweaksReloaded(r.unwrap_or_default()),
            ),
            Task::perform(
                async {
                    tokio::time::sleep(Duration::from_millis(120)).await;
                },
                |_| Message::SyncDwmTitlebar,
            ),
        ];

        if initial_tab == Tab::Debloater {
            tasks.push(Task::done(Message::ScanBloatware));
        }

        (app, Task::batch(tasks))
    }

    pub fn title(&self) -> String {
        if self.sys_info.is_admin {
            "Reforge — Administrator".into()
        } else {
            "Reforge".into()
        }
    }

    pub fn theme(&self) -> Theme {
        theme::iced_theme(self.config.theme)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.installing_active || self.updates_running {
            iced::time::every(Duration::from_millis(75)).map(|_| Message::PollStreams)
        } else {
            Subscription::none()
        }
    }

    pub fn is_catalog_app_installed(&self, app: &CatalogApp) -> bool {
        self.installed_apps.iter().any(|inst| {
            let lower = inst.name.to_lowercase();
            app.detect_keywords
                .iter()
                .any(|kw| lower.contains(&kw.to_lowercase()))
        })
    }

    fn set_toast(&mut self, msg: impl Into<String>, kind: ToastKind) {
        self.toast = Some(Toast {
            message: msg.into(),
            kind,
        });
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // ── NAVIGATION & GLOBAL ──────────────────────────────────────────
            Message::SelectTab(tab) => {
                self.active_tab = tab;
                if tab == Tab::Debloater && !self.bloat_loaded_once && !self.bloat_scanning {
                    return Task::done(Message::ScanBloatware);
                }
                Task::none()
            }
            Message::CycleTheme => {
                self.config.theme = self.config.theme.cycle();
                config::save_config(&self.config);
                winapi::apply_dwm_titlebar_theme(self.config.theme.is_dark());
                Task::none()
            }
            Message::SetTheme(mode) => {
                self.config.theme = mode;
                config::save_config(&self.config);
                winapi::apply_dwm_titlebar_theme(self.config.theme.is_dark());
                Task::none()
            }
            Message::ToggleLanguage => {
                self.config.lang = self.config.lang.toggle();
                config::save_config(&self.config);
                Task::none()
            }
            Message::SetLanguage(lang) => {
                self.config.lang = lang;
                config::save_config(&self.config);
                Task::none()
            }
            Message::SyncDwmTitlebar => {
                winapi::apply_dwm_titlebar_theme(self.config.theme.is_dark());
                Task::none()
            }
            Message::ElevateAsAdmin => {
                match winapi::relaunch_as_admin() {
                    Ok(()) => std::process::exit(0),
                    Err(e) => self.set_toast(e, ToastKind::Error),
                }
                Task::none()
            }
            Message::DismissToast => {
                self.toast = None;
                Task::none()
            }
            Message::OpenPath(path) => {
                winapi::open_folder_in_explorer(&path);
                Task::none()
            }
            Message::OpenUrl(url) => {
                winapi::open_url(&url);
                Task::none()
            }
            Message::PollStreams => {
                if let Some(rx_arc) = &self.install_rx {
                    if let Ok(mut rx) = rx_arc.lock() {
                        while let Ok(ev) = rx.try_recv() {
                            if let Some(existing) = self
                                .install_queue
                                .iter_mut()
                                .find(|t| t.app_id == ev.app_id)
                            {
                                existing.stage = ev.stage;
                            } else {
                                self.install_queue.push(InstallTaskState {
                                    app_id: ev.app_id,
                                    app_name: ev.app_name,
                                    stage: ev.stage,
                                });
                            }
                        }
                    }
                }
                if let Some(rx_arc) = &self.update_rx {
                    if let Ok(mut rx) = rx_arc.lock() {
                        while let Ok(ev) = rx.try_recv() {
                            if let Some(item) = self.updates.iter_mut().find(|u| u.id == ev.id) {
                                item.status = ev.status;
                            }
                        }
                    }
                }
                Task::none()
            }

            // ── SYSTEM ───────────────────────────────────────────────────────
            Message::RefreshSystemInfo => Task::perform(
                tokio::task::spawn_blocking(winapi::collect_system_info),
                |r| Message::SystemInfoLoaded(r.unwrap_or_default()),
            ),
            Message::SystemInfoLoaded(info) => {
                self.sys_info = info;
                Task::none()
            }
            Message::CreateRestorePoint => {
                self.set_toast("Creating Windows System Restore Point...", ToastKind::Info);
                Task::perform(
                    winapi::create_restore_point("Reforge Pre-Optimization Backup"),
                    Message::RestorePointFinished,
                )
            }
            Message::RestorePointFinished(res) => {
                match res {
                    Ok(msg) => self.set_toast(msg, ToastKind::Success),
                    Err(e) => self.set_toast(e, ToastKind::Error),
                }
                Task::none()
            }
            Message::BootstrapWinget => {
                self.winget_bootstrapping = true;
                self.set_toast("Downloading Winget (App Installer) from Microsoft GitHub...", ToastKind::Info);
                Task::perform(installer::bootstrap_winget(), Message::BootstrapWingetFinished)
            }
            Message::BootstrapWingetFinished(res) => {
                self.winget_bootstrapping = false;
                match res {
                    Ok(msg) => {
                        self.set_toast(msg, ToastKind::Success);
                        return Task::done(Message::RefreshSystemInfo);
                    }
                    Err(e) => self.set_toast(e, ToastKind::Error),
                }
                Task::none()
            }

            // ── INSTALLER ────────────────────────────────────────────────────
            Message::InstallerSearchChanged(q) => {
                self.installer_search = q;
                Task::none()
            }
            Message::InstallerCategoryChanged(cat) => {
                self.installer_category = cat;
                Task::none()
            }
            Message::ToggleCatalogApp(id) => {
                if !self.selected_catalog_ids.remove(id) {
                    self.selected_catalog_ids.insert(id);
                }
                Task::none()
            }
            Message::ApplyInstallPreset(preset) => {
                for id in preset_app_ids(preset) {
                    self.selected_catalog_ids.insert(id);
                }
                Task::none()
            }
            Message::ClearInstallSelection => {
                self.selected_catalog_ids.clear();
                self.selected_winget_ids.clear();
                Task::none()
            }
            Message::SearchWingetCatalog => {
                let q = self.installer_search.trim().to_string();
                if q.len() < 2 {
                    return Task::none();
                }
                self.winget_searching = true;
                Task::perform(
                    async move { installer::search_winget_catalog(&q).await },
                    Message::WingetSearchFinished,
                )
            }
            Message::WingetSearchFinished(res) => {
                self.winget_searching = false;
                match res {
                    Ok(items) => {
                        if items.is_empty() {
                            self.set_toast("No Winget packages matched that query.", ToastKind::Info);
                        }
                        self.winget_results = items;
                    }
                    Err(e) => self.set_toast(e, ToastKind::Error),
                }
                Task::none()
            }
            Message::ClearWingetResults => {
                self.winget_results.clear();
                Task::none()
            }
            Message::ToggleWingetApp(id, name) => {
                if self.selected_winget_ids.remove(&id).is_none() {
                    self.selected_winget_ids.insert(id, name);
                }
                Task::none()
            }
            Message::StartInstallBatch => {
                if self.installing_active {
                    return Task::none();
                }
                let mut jobs: Vec<InstallJob> = catalog()
                    .iter()
                    .filter(|a| self.selected_catalog_ids.contains(a.id))
                    .map(InstallJob::from)
                    .collect();

                for (wid, wname) in &self.selected_winget_ids {
                    jobs.push(InstallJob {
                        id: format!("wg_{wid}"),
                        name: wname.clone(),
                        winget_id: wid.clone(),
                        direct: None,
                    });
                }

                if jobs.is_empty() {
                    return Task::none();
                }

                self.install_queue = jobs
                    .iter()
                    .map(|j| InstallTaskState {
                        app_id: j.id.clone(),
                        app_name: j.name.clone(),
                        stage: InstallStage::Queued,
                    })
                    .collect();
                self.selected_catalog_ids.clear();
                self.selected_winget_ids.clear();
                self.installing_active = true;

                let (tx, rx) = mpsc::unbounded_channel();
                self.install_rx = Some(Arc::new(Mutex::new(rx)));

                let conc = self.config.max_concurrent_installs;
                let cleanup = self.config.auto_cleanup_downloads;
                Task::perform(
                    installer::run_install_batch(jobs, conc, cleanup, tx),
                    |_| Message::InstallBatchDone,
                )
            }
            Message::InstallBatchDone => {
                // Drain any final messages before clearing active state
                let _ = self.update(Message::PollStreams);
                self.installing_active = false;
                self.install_rx = None;
                let ok_count = self
                    .install_queue
                    .iter()
                    .filter(|t| matches!(t.stage, InstallStage::Done { .. }))
                    .count();
                let fail_count = self
                    .install_queue
                    .iter()
                    .filter(|t| matches!(t.stage, InstallStage::Failed(_)))
                    .count();
                self.set_toast(
                    format!("Install batch finished: {ok_count} installed, {fail_count} failed."),
                    if fail_count == 0 {
                        ToastKind::Success
                    } else {
                        ToastKind::Info
                    },
                );
                Task::done(Message::RefreshInstalledApps)
            }
            Message::ClearInstallQueue => {
                self.install_queue.clear();
                Task::none()
            }

            // ── UNINSTALLER ──────────────────────────────────────────────────
            Message::RefreshInstalledApps => {
                let inc = self.show_system_apps;
                Task::perform(
                    tokio::task::spawn_blocking(move || uninstaller::scan_installed_apps(inc)),
                    |r| Message::InstalledAppsLoaded(r.unwrap_or_default()),
                )
            }
            Message::InstalledAppsLoaded(apps) => {
                self.installed_apps = apps;
                Task::none()
            }
            Message::UninstallerSearchChanged(q) => {
                self.uninstaller_search = q;
                Task::none()
            }
            Message::ToggleSystemApps(val) => {
                self.show_system_apps = val;
                Task::done(Message::RefreshInstalledApps)
            }
            Message::UninstallApp(app) => {
                self.uninstalling_id = Some(app.id.clone());
                let quiet = self.config.prefer_quiet_uninstall;
                Task::perform(
                    async move {
                        let res = uninstaller::run_native_uninstaller(&app, quiet).await;
                        (app, res)
                    },
                    |(app, res)| Message::UninstallFinished(app, res),
                )
            }
            Message::UninstallFinished(app, res) => {
                self.uninstalling_id = None;
                match res {
                    Ok(()) => self.set_toast(
                        format!("{} uninstalled — scanning leftover traces...", app.name),
                        ToastKind::Success,
                    ),
                    Err(e) => self.set_toast(e, ToastKind::Error),
                }
                Task::batch([
                    Task::done(Message::RefreshInstalledApps),
                    Task::done(Message::DeepScanApp(app)),
                ])
            }
            Message::ForceRemoveApp(app) => {
                match uninstaller::force_remove_uninstall_entry(&app) {
                    Ok(()) => {
                        self.set_toast(
                            format!("Removed uninstall entry for {}. Scanning traces...", app.name),
                            ToastKind::Success,
                        );
                        return Task::batch([
                            Task::done(Message::RefreshInstalledApps),
                            Task::done(Message::DeepScanApp(app)),
                        ]);
                    }
                    Err(e) => self.set_toast(e, ToastKind::Error),
                }
                Task::none()
            }
            Message::DeepScanApp(app) => {
                self.trace_target_app = Some(app.clone());
                self.detected_traces.clear();
                self.traces_scanning = true;
                Task::perform(
                    tokio::task::spawn_blocking(move || uninstaller::deep_scan_traces(&app)),
                    |r| Message::DeepScanFinished(r.unwrap_or_default()),
                )
            }
            Message::DeepScanFinished(traces) => {
                self.traces_scanning = false;
                self.detected_traces = traces;
                Task::none()
            }
            Message::ToggleTrace(id) => {
                if let Some(t) = self.detected_traces.iter_mut().find(|x| x.id == id) {
                    t.selected = !t.selected;
                }
                Task::none()
            }
            Message::ToggleAllTraces(val) => {
                for t in &mut self.detected_traces {
                    t.selected = val;
                }
                Task::none()
            }
            Message::SelectHighConfidenceTraces => {
                for t in &mut self.detected_traces {
                    t.selected = t.confidence == TraceConfidence::High;
                }
                Task::none()
            }
            Message::DeleteSelectedTraces => {
                let traces = self.detected_traces.clone();
                Task::perform(
                    tokio::task::spawn_blocking(move || uninstaller::delete_selected_traces(&traces)),
                    |r| Message::TracesDeleted(r.unwrap_or_default()),
                )
            }
            Message::TracesDeleted(report) => {
                self.set_toast(
                    format!(
                        "Removed {} leftover traces ({} freed, {} locked/failed).",
                        report.deleted_count,
                        winapi::format_bytes(report.freed_bytes),
                        report.failed_count
                    ),
                    if report.failed_count == 0 {
                        ToastKind::Success
                    } else {
                        ToastKind::Info
                    },
                );
                if let Some(target) = self.trace_target_app.clone() {
                    return Task::batch([
                        Task::done(Message::RefreshInstalledApps),
                        Task::done(Message::DeepScanApp(target)),
                    ]);
                }
                Task::none()
            }
            Message::CloseTraceReview => {
                self.trace_target_app = None;
                self.detected_traces.clear();
                Task::none()
            }

            // ── UPDATER ──────────────────────────────────────────────────────
            Message::CheckUpdates => {
                self.updates_checking = true;
                Task::perform(updater::check_for_updates(), Message::UpdatesChecked)
            }
            Message::UpdatesChecked(res) => {
                self.updates_checking = false;
                self.updates_checked_once = true;
                match res {
                    Ok(list) => self.updates = list,
                    Err(e) => self.set_toast(e, ToastKind::Error),
                }
                Task::none()
            }
            Message::ToggleUpdateSelection(id) => {
                if let Some(u) = self.updates.iter_mut().find(|x| x.id == id) {
                    u.selected = !u.selected;
                }
                Task::none()
            }
            Message::RunSelectedUpdates => {
                let ids: Vec<String> = self
                    .updates
                    .iter()
                    .filter(|u| u.selected && u.status == UpdateItemStatus::Available)
                    .map(|u| u.id.clone())
                    .collect();
                self.start_updates(ids)
            }
            Message::RunSingleUpdate(id) => self.start_updates(vec![id]),
            Message::UpdatesBatchDone => {
                let _ = self.update(Message::PollStreams);
                self.updates_running = false;
                self.update_rx = None;
                self.set_toast("Update batch completed.", ToastKind::Success);
                Task::done(Message::RefreshInstalledApps)
            }

            // ── CLEANER ──────────────────────────────────────────────────────
            Message::ScanJunk => {
                self.cleaner_scanning = true;
                let prev: Vec<(CleanCategoryId, bool)> = self
                    .clean_scan
                    .categories
                    .iter()
                    .map(|c| (c.id, c.selected))
                    .collect();
                Task::perform(
                    tokio::task::spawn_blocking(move || {
                        cleaner::scan_system_junk(if prev.is_empty() { None } else { Some(&prev) })
                    }),
                    |r| Message::JunkScanned(r.unwrap_or_default()),
                )
            }
            Message::JunkScanned(res) => {
                self.cleaner_scanning = false;
                self.clean_scan = res;
                Task::none()
            }
            Message::ToggleCleanCategory(id) => {
                if let Some(c) = self.clean_scan.categories.iter_mut().find(|x| x.id == id) {
                    c.selected = !c.selected;
                }
                Task::none()
            }
            Message::CleanSelectedJunk => {
                self.cleaner_cleaning = true;
                let cats = self.clean_scan.categories.clone();
                Task::perform(
                    tokio::task::spawn_blocking(move || cleaner::clean_selected_categories(&cats)),
                    |r| Message::JunkCleaned(r.unwrap_or_default()),
                )
            }
            Message::JunkCleaned(res) => {
                self.cleaner_cleaning = false;
                self.set_toast(
                    format!(
                        "Freed {} across {} files in {} ms.",
                        winapi::format_bytes(res.freed_bytes),
                        res.deleted_files,
                        res.elapsed_ms
                    ),
                    ToastKind::Success,
                );
                self.last_clean_exec = Some(res);
                Task::batch([
                    Task::done(Message::ScanJunk),
                    Task::done(Message::RefreshSystemInfo),
                ])
            }

            // ── DEBLOATER ────────────────────────────────────────────────────
            Message::ScanBloatware => {
                self.bloat_scanning = true;
                Task::perform(debloater::scan_bloatware(), Message::BloatwareScanned)
            }
            Message::BloatwareScanned(res) => {
                self.bloat_scanning = false;
                self.bloat_loaded_once = true;
                match res {
                    Ok(list) => self.bloat_packages = list,
                    Err(e) => self.set_toast(e, ToastKind::Error),
                }
                Task::none()
            }
            Message::ToggleBloatPackage(full_name) => {
                if let Some(p) = self
                    .bloat_packages
                    .iter_mut()
                    .find(|x| x.package_full_name == full_name)
                {
                    p.selected = !p.selected;
                }
                Task::none()
            }
            Message::SelectSafeBloatOnly => {
                for p in &mut self.bloat_packages {
                    p.selected = p.safety == BloatSafety::Safe;
                }
                Task::none()
            }
            Message::RemoveSelectedBloat => {
                let selected: Vec<BloatPackage> = self
                    .bloat_packages
                    .iter()
                    .filter(|b| b.selected)
                    .cloned()
                    .collect();
                if selected.is_empty() {
                    return Task::none();
                }
                self.bloat_removing = true;
                Task::perform(
                    debloater::remove_bloat_packages(selected),
                    |(ok, fail)| Message::BloatRemoved(ok, fail),
                )
            }
            Message::BloatRemoved(ok, fail) => {
                self.bloat_removing = false;
                self.set_toast(
                    format!("Removed {ok} bloatware packages ({fail} failed)."),
                    if fail == 0 {
                        ToastKind::Success
                    } else {
                        ToastKind::Info
                    },
                );
                Task::done(Message::ScanBloatware)
            }

            // ── STARTUP ──────────────────────────────────────────────────────
            Message::RefreshStartupItems => Task::perform(
                tokio::task::spawn_blocking(startup::scan_startup_items),
                |r| Message::StartupItemsLoaded(r.unwrap_or_default()),
            ),
            Message::StartupItemsLoaded(items) => {
                self.startup_items = items;
                Task::none()
            }
            Message::StartupSearchChanged(q) => {
                self.startup_search = q;
                Task::none()
            }
            Message::ToggleStartupItem(id, enable) => {
                if let Some(item) = self.startup_items.iter_mut().find(|i| i.id == id) {
                    match startup::toggle_startup_item(item, enable) {
                        Ok(()) => item.enabled = enable,
                        Err(e) => self.set_toast(e, ToastKind::Error),
                    }
                }
                Task::none()
            }
            Message::DeleteStartupItem(id) => {
                if let Some(pos) = self.startup_items.iter().position(|i| i.id == id) {
                    let item = self.startup_items[pos].clone();
                    match startup::delete_startup_item(&item) {
                        Ok(()) => {
                            self.startup_items.remove(pos);
                            self.set_toast(
                                format!("Removed '{}' from startup.", item.name),
                                ToastKind::Success,
                            );
                        }
                        Err(e) => self.set_toast(e, ToastKind::Error),
                    }
                }
                Task::none()
            }

            // ── TWEAKS ───────────────────────────────────────────────────────
            Message::TweakFilterChanged(filter) => {
                self.tweak_filter = filter;
                Task::none()
            }
            Message::ToggleTweak(id, enable) => {
                match tweaks::set_tweak(id, enable) {
                    Ok(()) => {
                        if let Some(t) = self.tweaks.iter_mut().find(|x| x.id == id) {
                            t.enabled = enable;
                        }
                    }
                    Err(e) => self.set_toast(e, ToastKind::Error),
                }
                Task::none()
            }
            Message::ApplyRecommendedTweaks => {
                let mut ok = 0;
                let mut err = 0;
                for t in &mut self.tweaks {
                    if t.recommended && !t.enabled {
                        match tweaks::set_tweak(t.id, true) {
                            Ok(()) => {
                                t.enabled = true;
                                ok += 1;
                            }
                            Err(_) => err += 1,
                        }
                    }
                }
                self.set_toast(
                    format!("Applied {ok} recommended tweaks ({err} required Admin)."),
                    if err == 0 {
                        ToastKind::Success
                    } else {
                        ToastKind::Info
                    },
                );
                Task::none()
            }
            Message::RestoreAllTweaks => {
                let mut ok = 0;
                for t in &mut self.tweaks {
                    if t.enabled && tweaks::set_tweak(t.id, false).is_ok() {
                        t.enabled = false;
                        ok += 1;
                    }
                }
                self.set_toast(
                    format!("Restored {ok} settings back to Windows defaults."),
                    ToastKind::Success,
                );
                Task::none()
            }
            Message::TweaksReloaded(list) => {
                self.tweaks = list;
                Task::none()
            }

            // ── SETTINGS / PORTABLE ──────────────────────────────────────────
            Message::SetMaxConcurrency(n) => {
                self.config.max_concurrent_installs = n.clamp(1, 8);
                config::save_config(&self.config);
                Task::none()
            }
            Message::SetAutoCleanup(v) => {
                self.config.auto_cleanup_downloads = v;
                config::save_config(&self.config);
                Task::none()
            }
            Message::SetPreferQuietUninstall(v) => {
                self.config.prefer_quiet_uninstall = v;
                config::save_config(&self.config);
                Task::none()
            }
            Message::ExportProfile => {
                let profile = ReforgeProfile {
                    version: "1.0.0".into(),
                    selected_app_ids: self
                        .selected_catalog_ids
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    enabled_tweaks: self
                        .tweaks
                        .iter()
                        .filter(|t| t.enabled)
                        .map(|t| format!("{:?}", t.id))
                        .collect(),
                };
                match config::export_profile(&profile) {
                    Ok(path) => self.set_toast(
                        format!("Exported portable profile to {}", path.display()),
                        ToastKind::Success,
                    ),
                    Err(e) => self.set_toast(e, ToastKind::Error),
                }
                Task::none()
            }
            Message::ImportProfile => {
                match config::import_profile() {
                    Ok(prof) => {
                        self.selected_catalog_ids.clear();
                        for app in catalog() {
                            if prof.selected_app_ids.iter().any(|x| x == app.id) {
                                self.selected_catalog_ids.insert(app.id);
                            }
                        }
                        self.set_toast(
                            format!(
                                "Imported {} apps into Installer selection.",
                                self.selected_catalog_ids.len()
                            ),
                            ToastKind::Success,
                        );
                    }
                    Err(e) => self.set_toast(e, ToastKind::Error),
                }
                Task::none()
            }
        }
    }

    fn start_updates(&mut self, ids: Vec<String>) -> Task<Message> {
        if ids.is_empty() || self.updates_running {
            return Task::none();
        }
        self.updates_running = true;
        let (tx, rx) = mpsc::unbounded_channel();
        self.update_rx = Some(Arc::new(Mutex::new(rx)));
        let conc = self.config.max_concurrent_installs.min(3);
        Task::perform(
            updater::run_updates_batch(ids, conc, tx),
            |_| Message::UpdatesBatchDone,
        )
    }

    pub fn view(&self) -> Element<'_, Message> {
        let p = theme::palette(self.config.theme);

        let content: Element<'_, Message> = match self.active_tab {
            Tab::Installer => views::installer::view(self, p),
            Tab::Updater => views::updater::view(self, p),
            Tab::Uninstaller => views::uninstaller::view(self, p),
            Tab::Cleaner => views::cleaner::view(self, p),
            Tab::Debloater => views::debloater::view(self, p),
            Tab::Startup => views::startup::view(self, p),
            Tab::Tweaks => views::tweaks::view(self, p),
            Tab::Settings => views::settings::view(self, p),
        };

        let shell = container(row![
            views::sidebar::view(self, p),
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(theme::canvas_style(p)),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::canvas_style(p));

        if let Some(toast) = &self.toast {
            let accent = match toast.kind {
                ToastKind::Info => p.accent,
                ToastKind::Success => p.success,
                ToastKind::Error => p.danger,
            };
            let toast_ic: Element<'_, Message> = match toast.kind {
                ToastKind::Success => icons::check(14.0, accent).into(),
                ToastKind::Error => icons::alert_triangle(14.0, accent).into(),
                ToastKind::Info => icons::sparkles(14.0, accent).into(),
            };

            let toast_bar = container(
                row![
                    toast_ic,
                    text(&toast.message)
                        .font(FONT_MEDIUM)
                        .size(12)
                        .color(p.text_primary),
                    button(icons::close(12.0, p.text_muted))
                        .padding([3, 5])
                        .style(theme::btn_ghost(p))
                        .on_press(Message::DismissToast),
                ]
                .spacing(10)
                .align_y(Vertical::Center),
            )
            .padding([10, 14])
            .style(theme::toast_style(p, accent));

            let overlay = container(
                column![
                    Space::new().height(Length::Fill),
                    row![Space::new().width(Length::Fill), toast_bar]
                        .padding(Padding::from(18.0)),
                ]
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill);

            stack![shell, overlay].into()
        } else {
            shell.into()
        }
    }
}
