use crate::core::winapi::hidden_std_cmd;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TweakCategory {
    Performance,
    Gaming,
    Network,
    PrivacyAndEssentials,
}

impl TweakCategory {
    pub const ALL: &'static [TweakCategory] = &[
        TweakCategory::Performance,
        TweakCategory::Gaming,
        TweakCategory::Network,
        TweakCategory::PrivacyAndEssentials,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TweakId {
    // Performance & FPS
    UltimatePowerPlan,
    HardwareGpuScheduling,
    Win32PrioritySeparation,
    UnparkCpuCores,
    DisablePowerThrottling,
    DisableMemoryCompression,
    DisableSysMain,
    DisableStartupDelay,
    DisableHibernation,
    // Gaming & Input Latency
    DisableGameDvr,
    EnableGameMode,
    GamingMmcssPriority,
    DisableFullscreenOpt,
    RawMouseAndNoStickyKeys,
    // Network & Ping
    DisableNetworkThrottling,
    DisableNagleAlgorithm,
    DisableDeliveryOptP2p,
    CloudflareDns,
    // Privacy & Post-Format Essentials
    DisableTelemetryAndDiagTrack,
    DisableBackgroundApps,
    DisableStartBingSearch,
    DisableAdsAndAutoBloat,
    ClassicContextMenu,
    ShowExtensionsAndHidden,
    EnableTaskbarEndTask,
    PreventForcedUpdateReboots,
}

#[derive(Debug, Clone)]
pub struct SystemTweak {
    pub id: TweakId,
    pub category: TweakCategory,
    pub name_en: &'static str,
    pub name_pt: &'static str,
    pub desc_en: &'static str,
    pub desc_pt: &'static str,
    pub recommended: bool,
    pub requires_admin: bool,
    pub enabled: bool,
}

fn read_u32(root: winreg::HKEY, path: &str, name: &str) -> Option<u32> {
    RegKey::predef(root)
        .open_subkey_with_flags(path, KEY_READ)
        .ok()?
        .get_value::<u32, _>(name)
        .ok()
}

fn read_str(root: winreg::HKEY, path: &str, name: &str) -> Option<String> {
    RegKey::predef(root)
        .open_subkey_with_flags(path, KEY_READ)
        .ok()?
        .get_value::<String, _>(name)
        .ok()
}

fn write_u32(root: winreg::HKEY, path: &str, name: &str, val: u32) -> Result<(), String> {
    let (key, _) = RegKey::predef(root)
        .create_subkey(path)
        .map_err(|e| format!("Admin privileges required: {e}"))?;
    key.set_value(name, &val).map_err(|e| e.to_string())
}

fn write_str(root: winreg::HKEY, path: &str, name: &str, val: &str) -> Result<(), String> {
    let (key, _) = RegKey::predef(root)
        .create_subkey(path)
        .map_err(|e| format!("Admin privileges required: {e}"))?;
    key.set_value(name, &val).map_err(|e| e.to_string())
}

pub fn detect_tweak_state(id: TweakId) -> bool {
    match id {
        TweakId::UltimatePowerPlan => hidden_std_cmd("powercfg")
            .arg("/getactivescheme")
            .output()
            .map(|o| {
                let s = String::from_utf8_lossy(&o.stdout).to_lowercase();
                s.contains("8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c")
                    || s.contains("e9a42b02-d5df-448d-aa00-03f14749eb61")
                    || s.contains("ultimate")
                    || s.contains("desempenho máximo")
                    || s.contains("alto desempenho")
            })
            .unwrap_or(false),
        TweakId::HardwareGpuScheduling => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers",
                "HwSchMode",
            ) == Some(2)
        }
        TweakId::Win32PrioritySeparation => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SYSTEM\CurrentControlSet\Control\PriorityControl",
                "Win32PrioritySeparation",
            ) == Some(0x26)
        }
        TweakId::UnparkCpuCores => hidden_std_cmd("powercfg")
            .args([
                "/q",
                "SCHEME_CURRENT",
                "SUB_PROCESSOR",
                "CPMINCORES",
            ])
            .output()
            .map(|o| {
                let s = String::from_utf8_lossy(&o.stdout).to_lowercase();
                s.contains("0x00000064")
            })
            .unwrap_or(false),
        TweakId::DisablePowerThrottling => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SYSTEM\CurrentControlSet\Control\Power\PowerThrottling",
                "PowerThrottlingOff",
            ) == Some(1)
        }
        TweakId::DisableMemoryCompression => hidden_std_cmd("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "(Get-MMAgent).MemoryCompression",
            ])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().eq_ignore_ascii_case("False"))
            .unwrap_or(false),
        TweakId::DisableSysMain => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SYSTEM\CurrentControlSet\Services\SysMain",
                "Start",
            ) == Some(4)
        }
        TweakId::DisableStartupDelay => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\Serialize",
                "StartupDelayInMSec",
            ) == Some(0)
        }
        TweakId::DisableHibernation => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SYSTEM\CurrentControlSet\Control\Power",
                "HibernateEnabled",
            ) == Some(0)
        }
        TweakId::DisableGameDvr => {
            read_u32(
                HKEY_CURRENT_USER,
                r"System\GameConfigStore",
                "GameDVR_Enabled",
            ) == Some(0)
        }
        TweakId::EnableGameMode => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\GameBar",
                "AutoGameModeEnabled",
            ) == Some(1)
        }
        TweakId::GamingMmcssPriority => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games",
                "GPU Priority",
            ) == Some(8)
        }
        TweakId::DisableFullscreenOpt => {
            read_u32(
                HKEY_CURRENT_USER,
                r"System\GameConfigStore",
                "GameDVR_FSEBehaviorMode",
            ) == Some(2)
        }
        TweakId::RawMouseAndNoStickyKeys => {
            read_str(HKEY_CURRENT_USER, r"Control Panel\Mouse", "MouseSpeed").as_deref() == Some("0")
        }
        TweakId::DisableNetworkThrottling => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile",
                "NetworkThrottlingIndex",
            ) == Some(0xFFFF_FFFF)
        }
        TweakId::DisableNagleAlgorithm => {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            if let Ok(ifaces) = hklm.open_subkey_with_flags(
                r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces",
                KEY_READ,
            ) {
                for sub in ifaces.enum_keys().flatten() {
                    if let Ok(k) = ifaces.open_subkey_with_flags(&sub, KEY_READ) {
                        if k.get_value::<u32, _>("TCPNoDelay").ok() == Some(1) {
                            return true;
                        }
                    }
                }
            }
            false
        }
        TweakId::DisableDeliveryOptP2p => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization",
                "DODownloadMode",
            ) == Some(0)
        }
        TweakId::CloudflareDns => hidden_std_cmd("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "(Get-DnsClientServerAddress -AddressFamily IPv4 | Where-Object { $_.ServerAddresses -contains '1.1.1.1' }).Count -gt 0",
            ])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().eq_ignore_ascii_case("True"))
            .unwrap_or(false),
        TweakId::DisableTelemetryAndDiagTrack => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\Policies\Microsoft\Windows\DataCollection",
                "AllowTelemetry",
            ) == Some(0)
        }
        TweakId::DisableBackgroundApps => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications",
                "GlobalUserDisabled",
            ) == Some(1)
        }
        TweakId::DisableStartBingSearch => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Policies\Microsoft\Windows\Explorer",
                "DisableSearchBoxSuggestions",
            ) == Some(1)
        }
        TweakId::DisableAdsAndAutoBloat => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager",
                "SilentInstalledAppsEnabled",
            ) == Some(0)
        }
        TweakId::ClassicContextMenu => RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags(
                r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32",
                KEY_READ,
            )
            .is_ok(),
        TweakId::ShowExtensionsAndHidden => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                "HideFileExt",
            ) == Some(0)
        }
        TweakId::EnableTaskbarEndTask => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDeveloperSettings",
                "TaskbarEndTask",
            ) == Some(1)
        }
        TweakId::PreventForcedUpdateReboots => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU",
                "NoAutoRebootWithLoggedOnUsers",
            ) == Some(1)
        }
    }
}

pub fn load_all_tweaks() -> Vec<SystemTweak> {
    let defs: &[(
        TweakId,
        TweakCategory,
        &str,
        &str,
        &str,
        &str,
        bool,
        bool,
    )] = &[
        // Performance
        (
            TweakId::UltimatePowerPlan,
            TweakCategory::Performance,
            "Ultimate / High Performance Power Plan",
            "Plano de Energia Desempenho Máximo",
            "Activates Ultimate or High Performance scheme to prevent CPU downclocking",
            "Ativa Desempenho Máximo / Alto Desempenho eliminando quedas de clock da CPU",
            true,
            false,
        ),
        (
            TweakId::HardwareGpuScheduling,
            TweakCategory::Performance,
            "Hardware-Accelerated GPU Scheduling (HAGS)",
            "Agendamento de GPU Acelerado por Hardware (HAGS)",
            "Offloads GPU queue scheduling to dedicated VRAM processor for lower latency & DLSS FG",
            "Reduz latência de fila da GPU e habilita recursos como DLSS Frame Generation",
            true,
            true,
        ),
        (
            TweakId::Win32PrioritySeparation,
            TweakCategory::Performance,
            "Optimize CPU Thread Quantum (Win32Priority 0x26)",
            "Otimizar Quantum da CPU (Win32Priority 0x26)",
            "Short foreground-boosted CPU quanta for snappier responsiveness and tighter 1% low FPS",
            "Prioriza processos em primeiro plano com fatias curtas melhorando 1% low FPS",
            true,
            true,
        ),
        (
            TweakId::UnparkCpuCores,
            TweakCategory::Performance,
            "Unpark All CPU Cores (Disable Core Parking)",
            "Desestacionar Todos os Núcleos da CPU",
            "Keeps 100% of CPU cores active on AC power to eliminate wake-up frame spikes",
            "Mantém 100% dos núcleos ativos na energia evitando travamentos ao despertar cores",
            true,
            true,
        ),
        (
            TweakId::DisablePowerThrottling,
            TweakCategory::Performance,
            "Disable Windows Power Throttling",
            "Desativar Limitação de Energia (Power Throttling)",
            "Stops Windows from artificially throttling CPU frequency of active apps",
            "Impede o Windows de limitar a frequência da CPU de aplicativos ativos",
            true,
            true,
        ),
        (
            TweakId::DisableMemoryCompression,
            TweakCategory::Performance,
            "Disable RAM Memory Compression",
            "Desativar Compressão de Memória RAM",
            "Reduces CPU spikes & stutter on systems with 16 GB+ RAM (reboots cleanly)",
            "Reduz picos de uso de CPU e micro-stutter em PCs com 16 GB ou mais de RAM",
            false,
            true,
        ),
        (
            TweakId::DisableSysMain,
            TweakCategory::Performance,
            "Disable SysMain (Superfetch) Service",
            "Desativar Serviço SysMain (Superfetch)",
            "Stops continuous background disk caching that wastes SSD/NVMe I/O & RAM",
            "Interrompe pré-carregamento contínuo em disco que consome SSD e RAM à toa",
            true,
            true,
        ),
        (
            TweakId::DisableStartupDelay,
            TweakCategory::Performance,
            "Remove Boot App Startup Delay",
            "Remover Atraso de Inicialização do Windows",
            "Sets Explorer Serialize StartupDelayInMSec to 0 for faster boot readiness",
            "Define StartupDelayInMSec = 0 para apps carregarem instantaneamente no login",
            true,
            false,
        ),
        (
            TweakId::DisableHibernation,
            TweakCategory::Performance,
            "Disable Hibernation (Free hiberfil.sys)",
            "Desativar Hibernação (Liberar hiberfil.sys)",
            "Reclaims 6–16 GB of SSD space immediately (note: disables Fast Startup)",
            "Libera 6 a 16 GB no SSD imediatamente (obs: desativa Inicialização Rápida)",
            false,
            true,
        ),
        // Gaming
        (
            TweakId::DisableGameDvr,
            TweakCategory::Gaming,
            "Disable Game DVR Background Capture",
            "Desativar Gravação em Segundo Plano (Game DVR)",
            "Eliminates background Xbox capture overhead and frame-time spikes",
            "Elimina overhead de gravação na GPU e micro-stutters em jogos",
            true,
            false,
        ),
        (
            TweakId::EnableGameMode,
            TweakCategory::Gaming,
            "Enable Windows Game Mode",
            "Ativar Modo de Jogo do Windows",
            "Prioritizes active game threads and pauses Windows Update during gameplay",
            "Prioriza threads do jogo e pausa atualizações enquanto você joga",
            true,
            false,
        ),
        (
            TweakId::GamingMmcssPriority,
            TweakCategory::Gaming,
            "Gaming GPU & MMCSS High Scheduling Priority",
            "Prioridade Máxima de GPU e MMCSS para Jogos",
            "Sets SystemResponsiveness = 0, GPU Priority = 8 & High scheduling for game tasks",
            "Define SystemResponsiveness = 0, GPU Priority = 8 e agendamento Alto em jogos",
            true,
            true,
        ),
        (
            TweakId::DisableFullscreenOpt,
            TweakCategory::Gaming,
            "Disable Fullscreen Optimizations (Global)",
            "Desativar Otimizações de Tela Cheia (Global)",
            "Forces true exclusive fullscreen presentation to reduce DWM input lag",
            "Força tela cheia exclusiva reduzindo o input lag do compositor DWM",
            true,
            false,
        ),
        (
            TweakId::RawMouseAndNoStickyKeys,
            TweakCategory::Gaming,
            "Raw 1:1 Mouse Input & Disable Sticky Keys",
            "Mouse 1:1 Sem Aceleração + Desativar Teclas de Aderência",
            "Turns off pointer acceleration for consistent aim and kills Shift 5x popup",
            "Remove aceleração do mouse para mira 1:1 e bloqueia popup ao apertar Shift 5x",
            true,
            false,
        ),
        // Network
        (
            TweakId::DisableNetworkThrottling,
            TweakCategory::Network,
            "Disable Network Throttling Index",
            "Desativar Throttling de Rede do Windows",
            "Sets NetworkThrottlingIndex = 0xFFFFFFFF removing Windows' 10-packet/ms cap",
            "Remove o limite de 10 pacotes/ms do Windows melhorando ping e estabilidade",
            true,
            true,
        ),
        (
            TweakId::DisableNagleAlgorithm,
            TweakCategory::Network,
            "Disable Nagle's Algorithm (TCPNoDelay)",
            "Desativar Algoritmo de Nagle (TCPNoDelay)",
            "Sets TCPNoDelay & TcpAckFrequency = 1 on active adapters for lower packet latency",
            "Envia pacotes TCP imediatamente sem buffer reduzindo latência em jogos online",
            true,
            true,
        ),
        (
            TweakId::DisableDeliveryOptP2p,
            TweakCategory::Network,
            "Disable Windows Update P2P Uploads",
            "Desativar Uploads P2P do Windows Update",
            "Stops Windows from using your upload bandwidth to seed updates to other PCs",
            "Impede o Windows de consumir seu upload enviando updates para outros PCs",
            true,
            true,
        ),
        (
            TweakId::CloudflareDns,
            TweakCategory::Network,
            "Use Cloudflare 1.1.1.1 Low-Latency DNS",
            "Usar DNS Cloudflare 1.1.1.1 (Menor Latência)",
            "Configures 1.1.1.1 / 1.0.0.1 on active network adapters (reverts cleanly to DHCP)",
            "Aplica 1.1.1.1 / 1.0.0.1 nas placas ativas para resolução rápida (reversível para DHCP)",
            false,
            true,
        ),
        // Privacy & essentials
        (
            TweakId::DisableTelemetryAndDiagTrack,
            TweakCategory::PrivacyAndEssentials,
            "Disable Telemetry & DiagTrack Service",
            "Desativar Telemetria e Serviço DiagTrack",
            "Sets AllowTelemetry = 0 and stops/disables background Connected User Experiences",
            "Define telemetria no mínimo e desativa o serviço DiagTrack em segundo plano",
            true,
            true,
        ),
        (
            TweakId::DisableBackgroundApps,
            TweakCategory::PrivacyAndEssentials,
            "Disable Background UWP Apps",
            "Desativar Apps UWP em Segundo Plano",
            "Prevents closed Microsoft Store apps from waking up and using RAM/CPU",
            "Impede aplicativos fechados da Store de consumirem RAM e CPU",
            true,
            false,
        ),
        (
            TweakId::DisableStartBingSearch,
            TweakCategory::PrivacyAndEssentials,
            "Local-Only Start Menu (Disable Bing Search)",
            "Menu Iniciar 100% Local (Sem Busca Bing)",
            "Removes online Bing lag so Start Menu searches return local apps instantly",
            "Remove a busca online do Bing deixando o Menu Iniciar instantâneo",
            true,
            false,
        ),
        (
            TweakId::DisableAdsAndAutoBloat,
            TweakCategory::PrivacyAndEssentials,
            "Disable Windows Ads & Silent Bloat Installs",
            "Desativar Anúncios e Instalação Silenciosa de Bloat",
            "Stops Start/Lock Screen promos and prevents Windows auto-downloading OEM apps",
            "Remove anúncios do Windows e bloqueia download automático de bloatware",
            true,
            false,
        ),
        (
            TweakId::ClassicContextMenu,
            TweakCategory::PrivacyAndEssentials,
            "Classic Right-Click Context Menu (Win11)",
            "Menu de Contexto Clássico (Win11)",
            "Restores the instant full right-click menu without 'Show more options'",
            "Restaura o menu completo imediato no botão direito do Windows 11",
            true,
            false,
        ),
        (
            TweakId::ShowExtensionsAndHidden,
            TweakCategory::PrivacyAndEssentials,
            "Show File Extensions & Hidden Files",
            "Mostrar Extensões e Arquivos Ocultos",
            "Displays .exe/.json/.dll extensions and reveals AppData in File Explorer",
            "Exibe extensões reais de arquivos e mostra pastas ocultas como AppData",
            true,
            false,
        ),
        (
            TweakId::EnableTaskbarEndTask,
            TweakCategory::PrivacyAndEssentials,
            "Enable 'End Task' on Taskbar Right-Click",
            "Ativar 'Finalizar Tarefa' na Barra de Tarefas",
            "Kill frozen apps in one click directly from the Windows 11 taskbar",
            "Encerra aplicativos travados direto no botão direito da barra de tarefas",
            true,
            false,
        ),
        (
            TweakId::PreventForcedUpdateReboots,
            TweakCategory::PrivacyAndEssentials,
            "Prevent Forced Update Reboots While Active",
            "Bloquear Reinício Forçado do Windows Update",
            "Stops Windows Update from restarting your PC while you are signed in",
            "Impede o Windows Update de reiniciar o PC automaticamente durante seu uso",
            true,
            true,
        ),
    ];

    defs.iter()
        .map(|&(id, cat, ne, np, de, dp, rec, adm)| SystemTweak {
            id,
            category: cat,
            name_en: ne,
            name_pt: np,
            desc_en: de,
            desc_pt: dp,
            recommended: rec,
            requires_admin: adm,
            enabled: detect_tweak_state(id),
        })
        .collect()
}

/// Applies or reverts a tweak and verifies that Windows reflected the updated state.
pub fn set_tweak_verified(id: TweakId, enable: bool) -> Result<bool, String> {
    apply_tweak_raw(id, enable)?;
    let actual = detect_tweak_state(id);
    if actual != enable {
        return Err(
            "Setting applied, but Windows did not confirm the new state (Administrator rights or restart may be required)."
                .into(),
        );
    }
    Ok(actual)
}

fn apply_tweak_raw(id: TweakId, enable: bool) -> Result<(), String> {
    match id {
        TweakId::UltimatePowerPlan => {
            if enable {
                let dup = hidden_std_cmd("powercfg")
                    .args(["-duplicatescheme", "e9a42b02-d5df-448d-aa00-03f14749eb61"])
                    .output()
                    .ok();
                let activated_ult = dup.and_then(|o| {
                    let s = String::from_utf8_lossy(&o.stdout).to_string();
                    s.split_whitespace()
                        .find(|w| w.len() == 36 && w.chars().filter(|&c| c == '-').count() == 4)
                        .map(|guid| {
                            hidden_std_cmd("powercfg")
                                .args(["/setactive", guid])
                                .status()
                                .map(|st| st.success())
                                .unwrap_or(false)
                        })
                });
                if activated_ult != Some(true) {
                    let _ = hidden_std_cmd("powercfg")
                        .args(["/setactive", "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c"])
                        .output();
                }
            } else {
                let _ = hidden_std_cmd("powercfg")
                    .args(["/setactive", "381b4222-f694-41f0-9685-ff5bb260df2e"])
                    .output();
            }
            Ok(())
        }
        TweakId::HardwareGpuScheduling => write_u32(
            HKEY_LOCAL_MACHINE,
            r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers",
            "HwSchMode",
            if enable { 2 } else { 1 },
        ),
        TweakId::Win32PrioritySeparation => write_u32(
            HKEY_LOCAL_MACHINE,
            r"SYSTEM\CurrentControlSet\Control\PriorityControl",
            "Win32PrioritySeparation",
            if enable { 0x26 } else { 2 },
        ),
        TweakId::UnparkCpuCores => {
            let val = if enable { "100" } else { "5" };
            let out = hidden_std_cmd("powercfg")
                .args([
                    "/setacvalueindex",
                    "SCHEME_CURRENT",
                    "SUB_PROCESSOR",
                    "CPMINCORES",
                    val,
                ])
                .output()
                .map_err(|e| e.to_string())?;
            let _ = hidden_std_cmd("powercfg")
                .args(["/setactive", "SCHEME_CURRENT"])
                .output();
            if out.status.success() {
                Ok(())
            } else {
                Err("Failed to update CPU core parking index.".into())
            }
        }
        TweakId::DisablePowerThrottling => write_u32(
            HKEY_LOCAL_MACHINE,
            r"SYSTEM\CurrentControlSet\Control\Power\PowerThrottling",
            "PowerThrottlingOff",
            if enable { 1 } else { 0 },
        ),
        TweakId::DisableMemoryCompression => {
            let cmd = if enable {
                "Disable-MMAgent -MemoryCompression -ErrorAction Stop"
            } else {
                "Enable-MMAgent -MemoryCompression -ErrorAction Stop"
            };
            let out = hidden_std_cmd("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", cmd])
                .output()
                .map_err(|e| e.to_string())?;
            if out.status.success() {
                Ok(())
            } else {
                Err("Administrator privileges required to change Memory Compression.".into())
            }
        }
        TweakId::DisableSysMain => {
            write_u32(
                HKEY_LOCAL_MACHINE,
                r"SYSTEM\CurrentControlSet\Services\SysMain",
                "Start",
                if enable { 4 } else { 2 },
            )?;
            let action = if enable { "stop" } else { "start" };
            let _ = hidden_std_cmd("sc").args([action, "SysMain"]).output();
            Ok(())
        }
        TweakId::DisableStartupDelay => {
            if enable {
                write_u32(
                    HKEY_CURRENT_USER,
                    r"Software\Microsoft\Windows\CurrentVersion\Explorer\Serialize",
                    "StartupDelayInMSec",
                    0,
                )
            } else {
                let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(
                    r"Software\Microsoft\Windows\CurrentVersion\Explorer\Serialize",
                );
                Ok(())
            }
        }
        TweakId::DisableHibernation => {
            let arg = if enable { "off" } else { "on" };
            let out = hidden_std_cmd("powercfg")
                .args(["/h", arg])
                .output()
                .map_err(|e| e.to_string())?;
            if out.status.success() {
                Ok(())
            } else {
                Err("Administrator privileges required to toggle Hibernation.".into())
            }
        }
        TweakId::DisableGameDvr => {
            let v = if enable { 0 } else { 1 };
            write_u32(HKEY_CURRENT_USER, r"System\GameConfigStore", "GameDVR_Enabled", v)?;
            let _ = write_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\GameDVR",
                "AppCaptureEnabled",
                v,
            );
            Ok(())
        }
        TweakId::EnableGameMode => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\GameBar",
            "AutoGameModeEnabled",
            if enable { 1 } else { 0 },
        ),
        TweakId::GamingMmcssPriority => {
            let sys_prof =
                r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";
            let games = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games";
            if enable {
                write_u32(HKEY_LOCAL_MACHINE, sys_prof, "SystemResponsiveness", 0)?;
                write_u32(HKEY_LOCAL_MACHINE, games, "GPU Priority", 8)?;
                write_u32(HKEY_LOCAL_MACHINE, games, "Priority", 6)?;
                write_str(HKEY_LOCAL_MACHINE, games, "Scheduling Category", "High")?;
                write_str(HKEY_LOCAL_MACHINE, games, "SFIO Priority", "High")
            } else {
                write_u32(HKEY_LOCAL_MACHINE, sys_prof, "SystemResponsiveness", 20)?;
                write_u32(HKEY_LOCAL_MACHINE, games, "GPU Priority", 2)?;
                write_u32(HKEY_LOCAL_MACHINE, games, "Priority", 2)?;
                write_str(HKEY_LOCAL_MACHINE, games, "Scheduling Category", "Medium")?;
                write_str(HKEY_LOCAL_MACHINE, games, "SFIO Priority", "Normal")
            }
        }
        TweakId::DisableFullscreenOpt => {
            let path = r"System\GameConfigStore";
            if enable {
                write_u32(HKEY_CURRENT_USER, path, "GameDVR_FSEBehaviorMode", 2)?;
                write_u32(
                    HKEY_CURRENT_USER,
                    path,
                    "GameDVR_DXGIHonorFSEWindowsCompatible",
                    1,
                )?;
                write_u32(HKEY_CURRENT_USER, path, "GameDVR_HonorUserFSEBehaviorMode", 1)
            } else {
                write_u32(HKEY_CURRENT_USER, path, "GameDVR_FSEBehaviorMode", 0)?;
                write_u32(
                    HKEY_CURRENT_USER,
                    path,
                    "GameDVR_DXGIHonorFSEWindowsCompatible",
                    0,
                )?;
                write_u32(HKEY_CURRENT_USER, path, "GameDVR_HonorUserFSEBehaviorMode", 0)
            }
        }
        TweakId::RawMouseAndNoStickyKeys => {
            let mouse = r"Control Panel\Mouse";
            let sticky = r"Control Panel\Accessibility\StickyKeys";
            if enable {
                write_str(HKEY_CURRENT_USER, mouse, "MouseSpeed", "0")?;
                write_str(HKEY_CURRENT_USER, mouse, "MouseThreshold1", "0")?;
                write_str(HKEY_CURRENT_USER, mouse, "MouseThreshold2", "0")?;
                write_str(HKEY_CURRENT_USER, sticky, "Flags", "506")
            } else {
                write_str(HKEY_CURRENT_USER, mouse, "MouseSpeed", "1")?;
                write_str(HKEY_CURRENT_USER, mouse, "MouseThreshold1", "6")?;
                write_str(HKEY_CURRENT_USER, mouse, "MouseThreshold2", "10")?;
                write_str(HKEY_CURRENT_USER, sticky, "Flags", "510")
            }
        }
        TweakId::DisableNetworkThrottling => write_u32(
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile",
            "NetworkThrottlingIndex",
            if enable { 0xFFFF_FFFF } else { 10 },
        ),
        TweakId::DisableNagleAlgorithm => {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let base = r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces";
            let ifaces = hklm
                .open_subkey_with_flags(base, KEY_READ)
                .map_err(|e| format!("Admin privileges required: {e}"))?;
            let mut touched = 0;
            for sub in ifaces.enum_keys().flatten() {
                let full = format!("{base}\\{sub}");
                if let Ok(k) = hklm.open_subkey_with_flags(&full, KEY_READ | KEY_SET_VALUE) {
                    let has_ip = k.get_value::<String, _>("DhcpIPAddress").is_ok()
                        || k.get_value::<String, _>("IPAddress").is_ok();
                    if has_ip {
                        if enable {
                            let _ = k.set_value("TCPNoDelay", &1u32);
                            let _ = k.set_value("TcpAckFrequency", &1u32);
                        } else {
                            let _ = k.delete_value("TCPNoDelay");
                            let _ = k.delete_value("TcpAckFrequency");
                        }
                        touched += 1;
                    }
                }
            }
            if touched == 0 {
                Err("No active TCP/IP network interfaces could be modified (run as Admin).".into())
            } else {
                Ok(())
            }
        }
        TweakId::DisableDeliveryOptP2p => write_u32(
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization",
            "DODownloadMode",
            if enable { 0 } else { 1 },
        ),
        TweakId::CloudflareDns => {
            let ps = if enable {
                "Get-NetAdapter | Where-Object { $_.Status -eq 'Up' } | ForEach-Object { Set-DnsClientServerAddress -InterfaceIndex $_.ifIndex -ServerAddresses ('1.1.1.1','1.0.0.1') }"
            } else {
                "Get-NetAdapter | Where-Object { $_.Status -eq 'Up' } | ForEach-Object { Set-DnsClientServerAddress -InterfaceIndex $_.ifIndex -ResetServerAddresses }"
            };
            let out = hidden_std_cmd("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", ps])
                .output()
                .map_err(|e| e.to_string())?;
            if out.status.success() {
                let _ = hidden_std_cmd("ipconfig").arg("/flushdns").output();
                Ok(())
            } else {
                Err("Administrator privileges required to modify DNS servers.".into())
            }
        }
        TweakId::DisableTelemetryAndDiagTrack => {
            write_u32(
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\Policies\Microsoft\Windows\DataCollection",
                "AllowTelemetry",
                if enable { 0 } else { 1 },
            )?;
            let _ = write_u32(
                HKEY_LOCAL_MACHINE,
                r"SYSTEM\CurrentControlSet\Services\DiagTrack",
                "Start",
                if enable { 4 } else { 2 },
            );
            let _ = hidden_std_cmd("sc")
                .args([if enable { "stop" } else { "start" }, "DiagTrack"])
                .output();
            Ok(())
        }
        TweakId::DisableBackgroundApps => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications",
            "GlobalUserDisabled",
            if enable { 1 } else { 0 },
        ),
        TweakId::DisableStartBingSearch => {
            write_u32(
                HKEY_CURRENT_USER,
                r"Software\Policies\Microsoft\Windows\Explorer",
                "DisableSearchBoxSuggestions",
                if enable { 1 } else { 0 },
            )?;
            let _ = write_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Search",
                "BingSearchEnabled",
                if enable { 0 } else { 1 },
            );
            Ok(())
        }
        TweakId::DisableAdsAndAutoBloat => {
            let v = if enable { 0 } else { 1 };
            let cdm = r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager";
            write_u32(HKEY_CURRENT_USER, cdm, "SilentInstalledAppsEnabled", v)?;
            let _ = write_u32(HKEY_CURRENT_USER, cdm, "OemPreInstalledAppsEnabled", v);
            let _ = write_u32(HKEY_CURRENT_USER, cdm, "PreInstalledAppsEnabled", v);
            let _ = write_u32(HKEY_CURRENT_USER, cdm, "SubscribedContent-338389Enabled", v);
            let _ = write_u32(HKEY_CURRENT_USER, cdm, "SubscribedContent-338388Enabled", v);
            let _ = write_u32(HKEY_CURRENT_USER, cdm, "SystemPaneSuggestionsEnabled", v);
            Ok(())
        }
        TweakId::ClassicContextMenu => {
            let clsid = r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}";
            let inproc = format!(r"{clsid}\InprocServer32");
            if enable {
                write_str(HKEY_CURRENT_USER, &inproc, "", "")
            } else {
                let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(clsid);
                Ok(())
            }
        }
        TweakId::ShowExtensionsAndHidden => {
            let adv = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
            write_u32(HKEY_CURRENT_USER, adv, "HideFileExt", if enable { 0 } else { 1 })?;
            write_u32(HKEY_CURRENT_USER, adv, "Hidden", if enable { 1 } else { 2 })
        }
        TweakId::EnableTaskbarEndTask => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDeveloperSettings",
            "TaskbarEndTask",
            if enable { 1 } else { 0 },
        ),
        TweakId::PreventForcedUpdateReboots => write_u32(
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU",
            "NoAutoRebootWithLoggedOnUsers",
            if enable { 1 } else { 0 },
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_all_26_high_impact_tweaks_across_categories() {
        let all = load_all_tweaks();
        assert_eq!(all.len(), 26);
        for &cat in TweakCategory::ALL {
            assert!(
                all.iter().any(|t| t.category == cat),
                "Missing tweaks in category {:?}",
                cat
            );
        }
    }
}
