use crate::config::{self, PortableConfig, ReforgeProfile};
use crate::core::app_icons::{self, RgbaIcon};
use crate::core::catalog::{catalog, preset_app_ids, AppCategory, CatalogApp, PresetKind};
use crate::core::cleaner::{self, CleanCategoryId, CleanExecResult, CleanScanResult};
use crate::core::debloater::{self, BloatPackage, BloatSafety, DeepDebloatAction};
use crate::core::installer::{
    self, InstallEvent, InstallJob, InstallStage, WingetSearchResult,
};
use crate::core::startup::{self, StartupItem};
use crate::core::tweaks::{self, SystemTweak, TweakCategory, TweakId};
use crate::core::uninstaller::{
    self, AppTrace, InstalledApp, TraceConfidence, TraceDeleteReport, UninstallOptions,
    UninstallProgressEvent, UninstallStage,
};
use crate::core::updater::{self, AppUpdate, UpdateItemStatus, UpdateProgressEvent};
use crate::core::winapi::{self, SystemInfo};
use crate::i18n::Lang;
use crate::ui::icons;
use crate::ui::motion::MotionState;
use crate::ui::theme::{self, ThemeMode, FONT_MEDIUM};
use crate::ui::views;
use iced::alignment::Vertical;
use iced::widget::image::Handle as ImageHandle;
use iced::widget::scrollable::Viewport;
use iced::widget::{button, column, container, row, stack, text, Space};
use iced::{window, Element, Length, Padding, Subscription, Task, Theme};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerFilter {
    All,
    NotInstalled,
    Installed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerSort {
    Featured,
    Name,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UninstallerSort {
    Name,
    Size,
    Date,
}

#[derive(Debug, Clone)]
pub struct InstallTaskState {
    pub app_id: String,
    pub app_name: String,
    pub stage: InstallStage,
}

#[derive(Debug, Clone)]
pub struct UninstallTaskState {
    pub app_id: String,
    pub app_name: String,
    pub stage: UninstallStage,
}

#[derive(Debug, Clone)]
pub enum ActiveModal {
    ConfirmUninstall {
        apps: Vec<InstalledApp>,
        close_processes: bool,
        force_hung: bool,
        scan_traces_after: bool,
        prefer_quiet: bool,
    },
    UninstallProgress {
        items: Vec<UninstallTaskState>,
        finished: bool,
        scan_traces_after: bool,
    },
    AppDetailsInstalled(InstalledApp),
    AppDetailsCatalog(&'static CatalogApp),
    ConfirmTweaks {
        apply_recommended: bool,
        affected_count: usize,
        create_restore_point: bool,
    },
    ConfirmDebloat {
        packages: Vec<BloatPackage>,
    },
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
    AnimationTick(Instant),
    HoverEnter(String),
    HoverExit(String),

    // Modals
    CloseModal,
    ModalToggleCloseProcs(bool),
    ModalToggleForceHung(bool),
    ModalToggleScanTracesAfter(bool),
    ModalToggleQuietUninstall(bool),
    ModalToggleRestorePoint(bool),

    // Real App Icons
    InstalledIconsLoaded(HashMap<String, RgbaIcon>),
    CatalogIconsStreamDone,
    StartupIconsLoaded(HashMap<String, RgbaIcon>),

    // System
    RefreshSystemInfo,
    SystemInfoLoaded(SystemInfo),
    CreateRestorePoint,
    RestorePointFinished(Result<String, String>),
    BootstrapWinget,
    BootstrapWingetFinished(Result<String, String>),

    // Installer (Unified Curated + 16,000+ Live WinGet Infinite Scroll)
    InstallerSearchChanged(String),
    InstallerCategoryChanged(AppCategory),
    InstallerFilterChanged(InstallerFilter),
    InstallerSortChanged(InstallerSort),
    InstallerScrolled(Viewport),
    InstallerLoadMore,
    ToggleCatalogApp(&'static str),
    OpenCatalogDetails(&'static CatalogApp),
    ApplyInstallPreset(PresetKind),
    ClearInstallSelection,
    WingetIndexLoaded(Result<Vec<WingetSearchResult>, String>),
    ToggleWingetApp(String, String),
    StartInstallBatch,
    CancelInstallBatch,
    InstallBatchDone,
    ClearInstallQueue,

    // Uninstaller
    RefreshInstalledApps,
    InstalledAppsLoaded(Vec<InstalledApp>),
    UninstallerSearchChanged(String),
    UninstallerSortChanged(UninstallerSort),
    ToggleSystemApps(bool),
    ToggleUninstallSelection(String),
    ClearUninstallSelection,
    OpenInstalledDetails(InstalledApp),
    PromptBulkUninstall,
    ConfirmBulkUninstall,
    UninstallBatchDone,
    ProceedToBatchTraceScan,
    ForceRemoveApp(InstalledApp),
    DeepScanSelectedApps,
    DeepScanApps(Vec<InstalledApp>),
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
    RetryFailedUpdates,
    UpdatesBatchDone,

    // Cleaner
    ScanJunk,
    JunkScanned(CleanScanResult),
    ToggleCleanCategory(CleanCategoryId),
    CleanSelectedJunk,
    JunkCleaned(CleanExecResult),

    // Debloater
    ScanBloatware,
    ToggleShowAllUwp(bool),
    BloatwareScanned(Result<Vec<BloatPackage>, String>),
    ToggleBloatPackage(String),
    SelectSafeBloatOnly,
    PromptDebloatConfirm,
    ExecuteConfirmedDebloat,
    BloatRemoved(usize, usize),
    RunDeepDebloat(DeepDebloatAction),
    DeepDebloatFinished(Result<String, String>),

    // Startup
    RefreshStartupItems,
    StartupItemsLoaded(Vec<StartupItem>),
    StartupSearchChanged(String),
    ToggleStartupItem(String, bool),
    DeleteStartupItem(String),

    // Tweaks
    TweakFilterChanged(Option<TweakCategory>),
    ToggleTweak(TweakId, bool),
    PromptTweaksConfirm(bool),
    ExecuteConfirmedTweaks,
    TweaksReloaded(Vec<SystemTweak>),

    // Settings
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
    pub motion: MotionState,
    pub modal: Option<ActiveModal>,
    pub toast: Option<Toast>,
    pub winget_bootstrapping: bool,

    // Real App Icon Handle Caches
    installed_icons: HashMap<String, ImageHandle>,
    catalog_icons: HashMap<String, ImageHandle>,
    startup_icons: HashMap<String, ImageHandle>,
    catalog_icons_streaming: bool,
    catalog_icon_rx: Option<Arc<Mutex<mpsc::UnboundedReceiver<(String, RgbaIcon)>>>>,

    // Installer (Unified Curated + 16,000+ Live WinGet Catalog)
    pub installer_search: String,
    pub installer_category: AppCategory,
    pub installer_filter: InstallerFilter,
    pub installer_sort: InstallerSort,
    pub installer_visible_limit: usize,
    pub selected_catalog_ids: HashSet<&'static str>,
    pub selected_winget_ids: HashMap<String, String>,
    pub winget_full_index: Vec<WingetSearchResult>,
    pub winget_matched_indices: Vec<usize>,
    pub winget_searching: bool,
    pub install_queue: Vec<InstallTaskState>,
    pub installing_active: bool,
    install_cancel: Option<Arc<AtomicBool>>,
    install_rx: Option<Arc<Mutex<mpsc::UnboundedReceiver<InstallEvent>>>>,

    // Uninstaller
    pub installed_apps: Vec<InstalledApp>,
    pub uninstaller_search: String,
    pub uninstaller_sort: UninstallerSort,
    pub show_system_apps: bool,
    pub selected_uninstall_ids: HashSet<String>,
    pub uninstalling_active: bool,
    last_uninstall_targets: Vec<InstalledApp>,
    uninstall_rx: Option<Arc<Mutex<mpsc::UnboundedReceiver<UninstallProgressEvent>>>>,
    pub trace_targets: Vec<InstalledApp>,
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
    pub show_all_uwp: bool,
    pub deep_debloat_busy: Option<DeepDebloatAction>,

    // Startup
    pub startup_items: Vec<StartupItem>,
    pub startup_search: String,

    // Tweaks
    pub tweaks: Vec<SystemTweak>,
    pub tweak_filter: Option<TweakCategory>,

    // CLI visual preview hook
    preview_modal_on_load: bool,
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        let mut cfg = config::load_config();
        let mut initial_tab = Tab::Installer;
        let mut initial_search = String::new();
        let mut preview_modal_on_load = false;
        let mut check_updates_on_load = false;
        let mut preview_select_installer = false;

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
                    "dark" | "vesper" => ThemeMode::Dark,
                    "gemini" | "legacydark" => ThemeMode::GeminiDark,
                    "light" => ThemeMode::Light,
                    _ => ThemeMode::System,
                };
            } else if let Some(l) = arg.strip_prefix("--lang=") {
                cfg.lang = match l.to_lowercase().as_str() {
                    "pt" | "pt-br" => Lang::PtBr,
                    _ => Lang::En,
                };
            } else if let Some(q) = arg.strip_prefix("--search=") {
                initial_search = q.to_string();
            } else if arg == "--preview-uninstall-modal" {
                initial_tab = Tab::Uninstaller;
                preview_modal_on_load = true;
            } else if arg == "--check-updates" {
                initial_tab = Tab::Updater;
                check_updates_on_load = true;
            } else if arg == "--preview-select" {
                preview_select_installer = true;
            }
        }

        let reduced = winapi::is_reduced_motion_enabled();

        let catalog_domains: Vec<(String, String)> = catalog()
            .iter()
            .map(|c| (c.id.to_string(), c.domain.to_string()))
            .collect();

        // Instant synchronous load of already-cached catalog icons (<5 ms on warm launches)
        let (cached_raw, missing_domains) = app_icons::load_disk_cached_web_icons(&catalog_domains);
        let mut catalog_icons: HashMap<String, ImageHandle> = HashMap::with_capacity(cached_raw.len());
        for (k, rgba) in cached_raw {
            catalog_icons.insert(k, rgba.into_handle());
        }

        let (icon_tx, icon_rx) = mpsc::unbounded_channel();
        let streaming = !missing_domains.is_empty();

        let app = Self {
            active_tab: initial_tab,
            config: cfg,
            sys_info: SystemInfo::default(),
            motion: MotionState::new(reduced),
            modal: None,
            toast: None,
            winget_bootstrapping: false,

            installed_icons: HashMap::new(),
            catalog_icons,
            startup_icons: HashMap::new(),
            catalog_icons_streaming: streaming,
            catalog_icon_rx: if streaming {
                Some(Arc::new(Mutex::new(icon_rx)))
            } else {
                None
            },

            installer_search: initial_search,
            installer_category: AppCategory::All,
            installer_filter: InstallerFilter::All,
            installer_sort: InstallerSort::Featured,
            installer_visible_limit: 60,
            selected_catalog_ids: HashSet::new(),
            selected_winget_ids: HashMap::new(),
            winget_full_index: Vec::new(),
            winget_matched_indices: Vec::new(),
            winget_searching: true,
            install_queue: Vec::new(),
            installing_active: false,
            install_cancel: None,
            install_rx: None,

            installed_apps: Vec::new(),
            uninstaller_search: String::new(),
            uninstaller_sort: UninstallerSort::Name,
            show_system_apps: false,
            selected_uninstall_ids: HashSet::new(),
            uninstalling_active: false,
            last_uninstall_targets: Vec::new(),
            uninstall_rx: None,
            trace_targets: Vec::new(),
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
            show_all_uwp: false,
            deep_debloat_busy: None,

            startup_items: Vec::new(),
            startup_search: String::new(),

            tweaks: Vec::new(),
            tweak_filter: None,
            preview_modal_on_load,
        };

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

        if streaming {
            tasks.push(Task::perform(
                app_icons::stream_missing_web_icons(missing_domains, icon_tx),
                |_| Message::CatalogIconsStreamDone,
            ));
        }

        // Automatically index all 16,000+ Microsoft WinGet packages in the background
        // (<15 ms on warm disk cache) so search & infinite scroll work seamlessly with zero extra clicks!
        tasks.push(Task::perform(
            installer::load_full_winget_index(),
            Message::WingetIndexLoaded,
        ));

        if initial_tab == Tab::Debloater {
            tasks.push(Task::done(Message::ScanBloatware));
        }
        if check_updates_on_load {
            tasks.push(Task::done(Message::CheckUpdates));
        }

        let mut app = app;
        if preview_select_installer {
            app.selected_catalog_ids.insert("claude_desktop");
            app.selected_catalog_ids.insert("lunar_client");
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

    fn has_live_spinner(&self) -> bool {
        self.installing_active
            || self.uninstalling_active
            || self.updates_checking
            || self.updates_running
            || self.cleaner_scanning
            || self.cleaner_cleaning
            || self.bloat_scanning
            || self.bloat_removing
            || self.deep_debloat_busy.is_some()
            || self.traces_scanning
            || self.winget_searching
            || self.winget_bootstrapping
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = Vec::new();
        if self.installing_active
            || self.updates_running
            || self.uninstalling_active
            || self.catalog_icons_streaming
        {
            subs.push(iced::time::every(Duration::from_millis(65)).map(|_| Message::PollStreams));
        }
        if self.motion.is_animating(self.has_live_spinner()) {
            subs.push(window::frames().map(Message::AnimationTick));
        }
        if subs.is_empty() {
            Subscription::none()
        } else {
            Subscription::batch(subs)
        }
    }

    // ── REAL APP ICON LOOKUP HELPERS ─────────────────────────────────────────

    pub fn installed_icon(&self, app_id: &str) -> Option<&ImageHandle> {
        self.installed_icons.get(app_id)
    }

    pub fn startup_icon(&self, item_id: &str) -> Option<&ImageHandle> {
        self.startup_icons.get(item_id)
    }

    /// Prefers locally extracted native Win32 logo when installed; falls back to cached vendor logo.
    pub fn catalog_icon(&self, item: &CatalogApp) -> Option<&ImageHandle> {
        if let Some(inst) = self.installed_apps.iter().find(|inst| {
            let lower = inst.name.to_lowercase();
            item.detect_keywords
                .iter()
                .any(|kw| lower.contains(&kw.to_lowercase()))
        }) {
            if let Some(h) = self.installed_icons.get(&inst.id) {
                return Some(h);
            }
        }
        self.catalog_icons.get(item.id)
    }

    pub fn winget_icon(&self, winget_id: &str, name: &str) -> Option<&ImageHandle> {
        if let Some(cat) = crate::core::catalog::find_by_winget_id(winget_id) {
            if let Some(h) = self.catalog_icon(cat) {
                return Some(h);
            }
        }
        if let Some(h) = self.catalog_icons.get(&format!("wg_ic_{winget_id}")) {
            return Some(h);
        }
        let nlow = name.to_lowercase();
        if nlow.len() >= 3 {
            if let Some(inst) = self.installed_apps.iter().find(|i| {
                let ilow = i.name.to_lowercase();
                ilow == nlow || ilow.starts_with(&format!("{nlow} "))
            }) {
                if let Some(h) = self.installed_icons.get(&inst.id) {
                    return Some(h);
                }
            }
        }
        None
    }

    fn refresh_unified_winget_matches(&mut self) {
        let curated_ids: HashSet<String> = catalog()
            .iter()
            .map(|c| c.winget_id.to_lowercase())
            .collect();
        let q = self.installer_search.trim().to_lowercase();

        let mut scored: Vec<(u8, usize)> = Vec::with_capacity(if q.is_empty() {
            self.winget_full_index.len()
        } else {
            256
        });

        for (idx, item) in self.winget_full_index.iter().enumerate() {
            let id_l = item.id.to_lowercase();
            if curated_ids.contains(&id_l) {
                continue;
            }
            if q.is_empty() {
                scored.push((0, idx));
            } else {
                let name_l = item.name.to_lowercase();
                let pub_l = item.publisher.to_lowercase();
                let desc_l = item.description.to_lowercase();
                let rank = if name_l == q || id_l == q {
                    0
                } else if name_l.starts_with(&q) || id_l.starts_with(&q) {
                    1
                } else if name_l.contains(&q) || id_l.contains(&q) {
                    2
                } else if pub_l.contains(&q) || desc_l.contains(&q) {
                    3
                } else {
                    continue;
                };
                scored.push((rank, idx));
            }
        }

        if self.installer_sort == InstallerSort::Name {
            scored.sort_by(|a, b| {
                a.0.cmp(&b.0).then_with(|| {
                    self.winget_full_index[a.1]
                        .name
                        .to_lowercase()
                        .cmp(&self.winget_full_index[b.1].name.to_lowercase())
                })
            });
        } else if !q.is_empty() {
            scored.sort_by_key(|&(rank, idx)| (rank, idx));
        }

        self.winget_matched_indices = scored.into_iter().map(|(_, idx)| idx).collect();
    }

    pub fn update_icon(&self, upd: &AppUpdate) -> Option<&ImageHandle> {
        if let Some(cid) = upd.catalog_id {
            if let Some(cat) = catalog().iter().find(|c| c.id == cid) {
                if let Some(h) = self.catalog_icon(cat) {
                    return Some(h);
                }
            }
        }
        let ulow = upd.name.to_lowercase();
        if let Some(inst) = self
            .installed_apps
            .iter()
            .find(|i| i.name.to_lowercase().contains(&ulow) || ulow.contains(&i.name.to_lowercase()))
        {
            if let Some(h) = self.installed_icons.get(&inst.id) {
                return Some(h);
            }
        }
        None
    }

    pub fn is_catalog_app_installed(&self, app: &CatalogApp) -> bool {
        self.installed_apps.iter().any(|inst| {
            let lower = inst.name.to_lowercase();
            app.detect_keywords
                .iter()
                .any(|kw| lower.contains(&kw.to_lowercase()))
        })
    }

    fn open_modal(&mut self, m: ActiveModal) {
        self.motion.clear_hovers();
        self.modal = Some(m);
        self.motion.trigger_modal_enter();
    }

    fn set_toast(&mut self, msg: impl Into<String>, kind: ToastKind) {
        self.toast = Some(Toast {
            message: msg.into(),
            kind,
        });
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // ── NAVIGATION & MOTION ──────────────────────────────────────────
            Message::SelectTab(tab) => {
                if self.active_tab != tab {
                    self.active_tab = tab;
                    self.motion.clear_hovers();
                    self.motion.trigger_tab_transition();
                }
                if tab == Tab::Debloater && !self.bloat_loaded_once && !self.bloat_scanning {
                    return Task::done(Message::ScanBloatware);
                }
                Task::none()
            }
            Message::AnimationTick(now) => {
                self.motion.tick(now);
                Task::none()
            }
            Message::HoverEnter(id) => {
                self.motion.set_hover(id, true);
                Task::none()
            }
            Message::HoverExit(id) => {
                self.motion.set_hover(id, false);
                Task::none()
            }
            Message::CycleTheme => {
                self.config.theme = self.config.theme.cycle();
                config::save_config(&self.config);
                winapi::apply_dwm_titlebar_theme(
                    self.config.theme.is_dark(),
                    self.config.theme.titlebar_rgb(),
                );
                Task::none()
            }
            Message::SetTheme(mode) => {
                self.config.theme = mode;
                config::save_config(&self.config);
                winapi::apply_dwm_titlebar_theme(
                    self.config.theme.is_dark(),
                    self.config.theme.titlebar_rgb(),
                );
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
                winapi::apply_dwm_titlebar_theme(
                    self.config.theme.is_dark(),
                    self.config.theme.titlebar_rgb(),
                );
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

            // ── MODALS ───────────────────────────────────────────────────────
            Message::CloseModal => {
                if !self.uninstalling_active {
                    self.modal = None;
                    self.motion.clear_hovers();
                }
                Task::none()
            }
            Message::ModalToggleCloseProcs(v) => {
                if let Some(ActiveModal::ConfirmUninstall {
                    close_processes, ..
                }) = &mut self.modal
                {
                    *close_processes = v;
                }
                Task::none()
            }
            Message::ModalToggleForceHung(v) => {
                if let Some(ActiveModal::ConfirmUninstall { force_hung, .. }) = &mut self.modal {
                    *force_hung = v;
                }
                Task::none()
            }
            Message::ModalToggleScanTracesAfter(v) => {
                if let Some(ActiveModal::ConfirmUninstall {
                    scan_traces_after, ..
                }) = &mut self.modal
                {
                    *scan_traces_after = v;
                }
                Task::none()
            }
            Message::ModalToggleQuietUninstall(v) => {
                if let Some(ActiveModal::ConfirmUninstall { prefer_quiet, .. }) = &mut self.modal {
                    *prefer_quiet = v;
                }
                Task::none()
            }
            Message::ModalToggleRestorePoint(v) => {
                if let Some(ActiveModal::ConfirmTweaks {
                    create_restore_point,
                    ..
                }) = &mut self.modal
                {
                    *create_restore_point = v;
                }
                Task::none()
            }

            // ── REAL APP ICONS ───────────────────────────────────────────────
            Message::InstalledIconsLoaded(map) => {
                for (k, rgba) in map {
                    self.installed_icons.insert(k, rgba.into_handle());
                }
                Task::none()
            }
            Message::CatalogIconsStreamDone => {
                let _ = self.update(Message::PollStreams);
                self.catalog_icons_streaming = false;
                self.catalog_icon_rx = None;
                Task::none()
            }
            Message::StartupIconsLoaded(map) => {
                for (k, rgba) in map {
                    self.startup_icons.insert(k, rgba.into_handle());
                }
                Task::none()
            }

            // ── BACKGROUND STREAMS ───────────────────────────────────────────
            Message::PollStreams => {
                if let Some(rx_arc) = &self.catalog_icon_rx {
                    if let Ok(mut rx) = rx_arc.lock() {
                        while let Ok((key, rgba)) = rx.try_recv() {
                            self.catalog_icons.insert(key, rgba.into_handle());
                        }
                    }
                }
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
                if let Some(rx_arc) = &self.uninstall_rx {
                    if let Ok(mut rx) = rx_arc.lock() {
                        while let Ok(ev) = rx.try_recv() {
                            if let Some(ActiveModal::UninstallProgress { items, .. }) =
                                &mut self.modal
                            {
                                if let Some(it) = items.iter_mut().find(|x| x.app_id == ev.app_id) {
                                    it.stage = ev.stage;
                                }
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
                self.set_toast("Creating System Restore Point...", ToastKind::Info);
                Task::perform(
                    winapi::create_restore_point("Reforge System Backup"),
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
                self.set_toast("Downloading WinGet from official Microsoft release...", ToastKind::Info);
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

            // ── INSTALLER (UNIFIED CURATED + 16,000+ INFINITE SCROLL) ────────
            Message::InstallerSearchChanged(q) => {
                self.installer_search = q;
                self.installer_visible_limit = 60;
                self.refresh_unified_winget_matches();
                Task::none()
            }
            Message::InstallerCategoryChanged(cat) => {
                self.installer_category = cat;
                self.installer_visible_limit = 60;
                self.motion.clear_hovers();
                Task::none()
            }
            Message::InstallerFilterChanged(f) => {
                self.installer_filter = f;
                self.installer_visible_limit = 60;
                self.motion.clear_hovers();
                Task::none()
            }
            Message::InstallerSortChanged(s) => {
                self.installer_sort = s;
                self.installer_visible_limit = 60;
                self.refresh_unified_winget_matches();
                Task::none()
            }
            Message::InstallerScrolled(vp) => {
                if vp.content_bounds().height > vp.bounds().height
                    && vp.relative_offset().y >= 0.80
                {
                    let max_total = catalog().len() + self.winget_matched_indices.len();
                    if self.installer_visible_limit < max_total {
                        self.installer_visible_limit =
                            (self.installer_visible_limit + 60).min(max_total);
                    }
                }
                Task::none()
            }
            Message::InstallerLoadMore => {
                let max_total = catalog().len() + self.winget_matched_indices.len();
                self.installer_visible_limit = (self.installer_visible_limit + 80).min(max_total);
                Task::none()
            }
            Message::ToggleCatalogApp(id) => {
                if !self.selected_catalog_ids.remove(id) {
                    self.selected_catalog_ids.insert(id);
                }
                Task::none()
            }
            Message::OpenCatalogDetails(app) => {
                self.open_modal(ActiveModal::AppDetailsCatalog(app));
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
            Message::WingetIndexLoaded(res) => {
                self.winget_searching = false;
                if let Ok(items) = res {
                    self.winget_full_index = items;
                    self.refresh_unified_winget_matches();
                }
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

                let cancel = Arc::new(AtomicBool::new(false));
                self.install_cancel = Some(Arc::clone(&cancel));
                let (tx, rx) = mpsc::unbounded_channel();
                self.install_rx = Some(Arc::new(Mutex::new(rx)));

                let conc = self.config.max_concurrent_installs;
                let cleanup = self.config.auto_cleanup_downloads;
                Task::perform(
                    installer::run_install_batch(jobs, conc, cleanup, cancel, tx),
                    |_| Message::InstallBatchDone,
                )
            }
            Message::CancelInstallBatch => {
                if let Some(flag) = &self.install_cancel {
                    flag.store(true, Ordering::Relaxed);
                    self.set_toast("Cancelling remaining downloads...", ToastKind::Info);
                }
                Task::none()
            }
            Message::InstallBatchDone => {
                let _ = self.update(Message::PollStreams);
                self.installing_active = false;
                self.install_rx = None;
                self.install_cancel = None;
                let ok = self
                    .install_queue
                    .iter()
                    .filter(|t| matches!(t.stage, InstallStage::Done { .. }))
                    .count();
                let fail = self
                    .install_queue
                    .iter()
                    .filter(|t| matches!(t.stage, InstallStage::Failed(_)))
                    .count();
                self.set_toast(
                    format!("Install batch finished: {ok} installed, {fail} failed."),
                    if fail == 0 {
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
                let icon_inputs: Vec<(String, String, String, String, String)> = apps
                    .iter()
                    .filter(|a| !self.installed_icons.contains_key(&a.id))
                    .map(|a| {
                        (
                            a.id.clone(),
                            a.name.clone(),
                            a.display_icon.clone(),
                            a.install_location.clone(),
                            a.uninstall_string.clone(),
                        )
                    })
                    .collect();

                // Prune any selections for apps no longer installed
                let valid_ids: HashSet<String> = apps.iter().map(|a| a.id.clone()).collect();
                self.selected_uninstall_ids
                    .retain(|id| valid_ids.contains(id));

                self.installed_apps = apps;

                if self.preview_modal_on_load && self.installed_apps.len() >= 3 {
                    self.preview_modal_on_load = false;
                    let sample: Vec<InstalledApp> =
                        self.installed_apps.iter().take(3).cloned().collect();
                    for a in &sample {
                        self.selected_uninstall_ids.insert(a.id.clone());
                    }
                    self.open_modal(ActiveModal::ConfirmUninstall {
                        apps: sample,
                        close_processes: true,
                        force_hung: false,
                        scan_traces_after: true,
                        prefer_quiet: self.config.prefer_quiet_uninstall,
                    });
                }

                if icon_inputs.is_empty() {
                    Task::none()
                } else {
                    Task::perform(
                        tokio::task::spawn_blocking(move || {
                            app_icons::extract_installed_icons_batch(&icon_inputs)
                        }),
                        |r| Message::InstalledIconsLoaded(r.unwrap_or_default()),
                    )
                }
            }
            Message::UninstallerSearchChanged(q) => {
                self.uninstaller_search = q;
                Task::none()
            }
            Message::UninstallerSortChanged(sort) => {
                self.uninstaller_sort = sort;
                Task::none()
            }
            Message::ToggleSystemApps(val) => {
                self.show_system_apps = val;
                Task::done(Message::RefreshInstalledApps)
            }
            Message::ToggleUninstallSelection(id) => {
                if !self.selected_uninstall_ids.remove(&id) {
                    self.selected_uninstall_ids.insert(id);
                }
                Task::none()
            }
            Message::ClearUninstallSelection => {
                self.selected_uninstall_ids.clear();
                Task::none()
            }
            Message::OpenInstalledDetails(app) => {
                self.open_modal(ActiveModal::AppDetailsInstalled(app));
                Task::none()
            }
            Message::PromptBulkUninstall => {
                let selected: Vec<InstalledApp> = self
                    .installed_apps
                    .iter()
                    .filter(|a| self.selected_uninstall_ids.contains(&a.id))
                    .cloned()
                    .collect();
                if selected.is_empty() {
                    return Task::none();
                }
                self.open_modal(ActiveModal::ConfirmUninstall {
                    apps: selected,
                    close_processes: true,
                    force_hung: false,
                    scan_traces_after: true,
                    prefer_quiet: self.config.prefer_quiet_uninstall,
                });
                Task::none()
            }
            Message::ConfirmBulkUninstall => {
                let Some(ActiveModal::ConfirmUninstall {
                    apps,
                    close_processes,
                    force_hung,
                    scan_traces_after,
                    prefer_quiet,
                }) = self.modal.take()
                else {
                    return Task::none();
                };

                let progress_items: Vec<UninstallTaskState> = apps
                    .iter()
                    .map(|a| UninstallTaskState {
                        app_id: a.id.clone(),
                        app_name: a.name.clone(),
                        stage: UninstallStage::Queued,
                    })
                    .collect();

                self.last_uninstall_targets = apps.clone();
                self.uninstalling_active = true;
                self.open_modal(ActiveModal::UninstallProgress {
                    items: progress_items,
                    finished: false,
                    scan_traces_after,
                });

                let (tx, rx) = mpsc::unbounded_channel();
                self.uninstall_rx = Some(Arc::new(Mutex::new(rx)));

                let opts = UninstallOptions {
                    close_processes,
                    force_terminate_if_hung: force_hung,
                    prefer_quiet,
                };

                Task::perform(
                    uninstaller::run_bulk_uninstall_lifecycle(apps, opts, tx),
                    |_| Message::UninstallBatchDone,
                )
            }
            Message::UninstallBatchDone => {
                let _ = self.update(Message::PollStreams);
                self.uninstalling_active = false;
                self.uninstall_rx = None;
                self.selected_uninstall_ids.clear();
                if let Some(ActiveModal::UninstallProgress { finished, .. }) = &mut self.modal {
                    *finished = true;
                }
                Task::done(Message::RefreshInstalledApps)
            }
            Message::ProceedToBatchTraceScan => {
                self.modal = None;
                let targets = self.last_uninstall_targets.clone();
                Task::done(Message::DeepScanApps(targets))
            }
            Message::ForceRemoveApp(app) => {
                self.modal = None;
                match uninstaller::force_remove_uninstall_entry(&app) {
                    Ok(()) => {
                        self.set_toast(
                            format!("Removed uninstall entry for {}.", app.name),
                            ToastKind::Success,
                        );
                        return Task::batch([
                            Task::done(Message::RefreshInstalledApps),
                            Task::done(Message::DeepScanApps(vec![app])),
                        ]);
                    }
                    Err(e) => self.set_toast(e, ToastKind::Error),
                }
                Task::none()
            }
            Message::DeepScanSelectedApps => {
                let selected: Vec<InstalledApp> = self
                    .installed_apps
                    .iter()
                    .filter(|a| self.selected_uninstall_ids.contains(&a.id))
                    .cloned()
                    .collect();
                if selected.is_empty() {
                    return Task::none();
                }
                Task::done(Message::DeepScanApps(selected))
            }
            Message::DeepScanApps(apps) => {
                self.modal = None;
                if apps.is_empty() {
                    return Task::none();
                }
                self.trace_targets = apps.clone();
                self.detected_traces.clear();
                self.traces_scanning = true;
                Task::perform(
                    tokio::task::spawn_blocking(move || uninstaller::deep_scan_traces_multi(&apps)),
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
                        "Removed {} traces ({} freed, {} skipped).",
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
                let targets = self.trace_targets.clone();
                Task::batch([
                    Task::done(Message::RefreshInstalledApps),
                    Task::done(Message::DeepScanApps(targets)),
                ])
            }
            Message::CloseTraceReview => {
                self.trace_targets.clear();
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
            Message::RetryFailedUpdates => {
                let ids: Vec<String> = self
                    .updates
                    .iter()
                    .filter(|u| matches!(u.status, UpdateItemStatus::Failed(_)))
                    .map(|u| u.id.clone())
                    .collect();
                self.start_updates(ids)
            }
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
                let include_all = self.show_all_uwp;
                Task::perform(
                    debloater::scan_bloatware(include_all),
                    Message::BloatwareScanned,
                )
            }
            Message::ToggleShowAllUwp(val) => {
                self.show_all_uwp = val;
                Task::done(Message::ScanBloatware)
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
            Message::PromptDebloatConfirm => {
                let selected: Vec<BloatPackage> = self
                    .bloat_packages
                    .iter()
                    .filter(|b| b.selected)
                    .cloned()
                    .collect();
                if !selected.is_empty() {
                    self.open_modal(ActiveModal::ConfirmDebloat { packages: selected });
                }
                Task::none()
            }
            Message::ExecuteConfirmedDebloat => {
                let Some(ActiveModal::ConfirmDebloat { packages }) = self.modal.take() else {
                    return Task::none();
                };
                self.bloat_removing = true;
                Task::perform(
                    debloater::remove_bloat_packages(packages),
                    |(ok, fail)| Message::BloatRemoved(ok, fail),
                )
            }
            Message::BloatRemoved(ok, fail) => {
                self.bloat_removing = false;
                self.set_toast(
                    format!("Removed {ok} packages ({fail} failed)."),
                    if fail == 0 {
                        ToastKind::Success
                    } else {
                        ToastKind::Info
                    },
                );
                Task::done(Message::ScanBloatware)
            }
            Message::RunDeepDebloat(action) => {
                if self.deep_debloat_busy.is_some() {
                    return Task::none();
                }
                self.deep_debloat_busy = Some(action);
                Task::perform(
                    debloater::run_deep_debloat_action(action),
                    Message::DeepDebloatFinished,
                )
            }
            Message::DeepDebloatFinished(res) => {
                self.deep_debloat_busy = None;
                match res {
                    Ok(msg) => self.set_toast(msg, ToastKind::Success),
                    Err(e) => self.set_toast(e, ToastKind::Error),
                }
                Task::done(Message::ScanBloatware)
            }

            // ── STARTUP ──────────────────────────────────────────────────────
            Message::RefreshStartupItems => Task::perform(
                tokio::task::spawn_blocking(startup::scan_startup_items),
                |r| Message::StartupItemsLoaded(r.unwrap_or_default()),
            ),
            Message::StartupItemsLoaded(items) => {
                let icon_inputs: Vec<(String, String)> = items
                    .iter()
                    .filter(|i| !self.startup_icons.contains_key(&i.id))
                    .map(|i| (i.id.clone(), i.command.clone()))
                    .collect();
                self.startup_items = items;
                if icon_inputs.is_empty() {
                    Task::none()
                } else {
                    Task::perform(
                        tokio::task::spawn_blocking(move || {
                            app_icons::extract_startup_icons_batch(&icon_inputs)
                        }),
                        |r| Message::StartupIconsLoaded(r.unwrap_or_default()),
                    )
                }
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
                match tweaks::set_tweak_verified(id, enable) {
                    Ok(actual) => {
                        if let Some(t) = self.tweaks.iter_mut().find(|x| x.id == id) {
                            t.enabled = actual;
                        }
                    }
                    Err(e) => self.set_toast(e, ToastKind::Error),
                }
                Task::none()
            }
            Message::PromptTweaksConfirm(apply_recommended) => {
                let count = if apply_recommended {
                    self.tweaks
                        .iter()
                        .filter(|t| t.recommended && !t.enabled)
                        .count()
                } else {
                    self.tweaks.iter().filter(|t| t.enabled).count()
                };
                if count == 0 {
                    self.set_toast(
                        if apply_recommended {
                            "All recommended optimizations are already enabled."
                        } else {
                            "All optimizations are already at Windows defaults."
                        },
                        ToastKind::Info,
                    );
                    return Task::none();
                }
                self.open_modal(ActiveModal::ConfirmTweaks {
                    apply_recommended,
                    affected_count: count,
                    create_restore_point: self.sys_info.is_admin,
                });
                Task::none()
            }
            Message::ExecuteConfirmedTweaks => {
                let Some(ActiveModal::ConfirmTweaks {
                    apply_recommended,
                    create_restore_point,
                    ..
                }) = self.modal.take()
                else {
                    return Task::none();
                };

                let mut ok = 0;
                let mut err = 0;
                for t in &mut self.tweaks {
                    let should_change = if apply_recommended {
                        t.recommended && !t.enabled
                    } else {
                        t.enabled
                    };
                    if should_change {
                        match tweaks::set_tweak_verified(t.id, apply_recommended) {
                            Ok(actual) => {
                                t.enabled = actual;
                                ok += 1;
                            }
                            Err(_) => err += 1,
                        }
                    }
                }

                self.set_toast(
                    if apply_recommended {
                        format!("Applied {ok} verified settings ({err} required Admin).")
                    } else {
                        format!("Restored {ok} settings to Windows defaults.")
                    },
                    if err == 0 {
                        ToastKind::Success
                    } else {
                        ToastKind::Info
                    },
                );

                if create_restore_point {
                    Task::done(Message::CreateRestorePoint)
                } else {
                    Task::none()
                }
            }
            Message::TweaksReloaded(list) => {
                self.tweaks = list;
                Task::none()
            }

            // ── SETTINGS ─────────────────────────────────────────────────────
            Message::SetMaxConcurrency(n) => {
                self.config.max_concurrent_installs = n.clamp(1, 6);
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

        let base = container(row![
            views::sidebar::view(self, p),
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(theme::canvas_style(p)),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::canvas_style(p));

        let mut layers: Vec<Element<'_, Message>> = vec![base.into()];

        if let Some(toast) = &self.toast {
            let accent = match toast.kind {
                ToastKind::Info => p.accent,
                ToastKind::Success => p.success,
                ToastKind::Error => p.danger,
            };
            let toast_ic: Element<'_, Message> = match toast.kind {
                ToastKind::Success => icons::check(13.0, accent).into(),
                ToastKind::Error => icons::alert_triangle(13.0, accent).into(),
                ToastKind::Info => icons::info(13.0, accent).into(),
            };

            let toast_bar = container(
                row![
                    toast_ic,
                    text(&toast.message)
                        .font(FONT_MEDIUM)
                        .size(12)
                        .color(p.text_primary),
                    button(icons::close(11.0, p.text_muted))
                        .padding([3, 4])
                        .style(theme::btn_ghost(p))
                        .on_press(Message::DismissToast),
                ]
                .spacing(9)
                .align_y(Vertical::Center),
            )
            .padding([9, 13])
            .style(theme::toast_style(p));

            layers.push(
                container(
                    column![
                        Space::new().height(Length::Fill),
                        row![Space::new().width(Length::Fill), toast_bar]
                            .padding(Padding::from(16.0)),
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            );
        }

        if let Some(modal) = &self.modal {
            layers.push(views::modals::view(self, modal, p));
        }

        stack(layers).into()
    }
}
