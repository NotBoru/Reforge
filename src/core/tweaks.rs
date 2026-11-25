use crate::core::winapi::hidden_std_cmd;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TweakCategory {
    Privacy,
    Performance,
    Gaming,
    Explorer,
    PowerAndSystem,
    Maintenance,
}

impl TweakCategory {
    pub const ALL: &'static [TweakCategory] = &[
        TweakCategory::Privacy,
        TweakCategory::Performance,
        TweakCategory::Gaming,
        TweakCategory::Explorer,
        TweakCategory::PowerAndSystem,
        TweakCategory::Maintenance,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TweakId {
    // Privacy
    DisableTelemetry,
    DisableAdvertisingId,
    DisableStartBingSearch,
    DisableTipsAndAds,
    DisableActivityHistory,
    DisableTailoredExperiences,
    // Performance
    HighPerformancePowerPlan,
    DisableBackgroundApps,
    DisableStartupDelay,
    DisableVisualTransparency,
    DisableDeliveryOptP2p,
    DisableHibernation,
    // Gaming
    DisableGameDvr,
    DisableMouseAccel,
    EnableGameMode,
    DisableNetworkThrottling,
    // Explorer & Interface
    ShowFileExtensions,
    ShowHiddenFiles,
    LaunchExplorerToThisPc,
    ClassicContextMenu,
    EnableTaskbarEndTask,
    AlignTaskbarLeft,
    HideCopilotAndWidgets,
    // Power & System
    DisableStickyKeysPopup,
    DisableFastStartup,
    DisableWerReporting,
    DisableAutoRebootUpdates,
    // Maintenance
    EnableStorageSense,
    DisableQuickAccessRecent,
    DisableSearchHighlights,
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
        .map_err(|e| format!("Permission denied (run Reforge as Admin): {e}"))?;
    key.set_value(name, &val).map_err(|e| e.to_string())
}

fn write_str(root: winreg::HKEY, path: &str, name: &str, val: &str) -> Result<(), String> {
    let (key, _) = RegKey::predef(root)
        .create_subkey(path)
        .map_err(|e| format!("Permission denied (run Reforge as Admin): {e}"))?;
    key.set_value(name, &val).map_err(|e| e.to_string())
}

pub fn detect_tweak_state(id: TweakId) -> bool {
    match id {
        TweakId::DisableTelemetry => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\Policies\Microsoft\Windows\DataCollection",
                "AllowTelemetry",
            ) == Some(0)
        }
        TweakId::DisableAdvertisingId => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo",
                "Enabled",
            ) == Some(0)
        }
        TweakId::DisableStartBingSearch => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Policies\Microsoft\Windows\Explorer",
                "DisableSearchBoxSuggestions",
            ) == Some(1)
        }
        TweakId::DisableTipsAndAds => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager",
                "SubscribedContent-338389Enabled",
            ) == Some(0)
        }
        TweakId::DisableActivityHistory => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\Policies\Microsoft\Windows\System",
                "EnableActivityFeed",
            ) == Some(0)
        }
        TweakId::DisableTailoredExperiences => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Privacy",
                "TailoredExperiencesWithDiagnosticDataEnabled",
            ) == Some(0)
        }
        TweakId::HighPerformancePowerPlan => hidden_std_cmd("powercfg")
            .arg("/getactivescheme")
            .output()
            .map(|o| {
                let s = String::from_utf8_lossy(&o.stdout).to_lowercase();
                s.contains("8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c")
                    || s.contains("e9a42b02-d5df-448d-aa00-03f14749eb61")
            })
            .unwrap_or(false),
        TweakId::DisableBackgroundApps => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications",
                "GlobalUserDisabled",
            ) == Some(1)
        }
        TweakId::DisableStartupDelay => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\Serialize",
                "StartupDelayInMSec",
            ) == Some(0)
        }
        TweakId::DisableVisualTransparency => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                "EnableTransparency",
            ) == Some(0)
        }
        TweakId::DisableDeliveryOptP2p => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization",
                "DODownloadMode",
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
        TweakId::DisableMouseAccel => {
            read_str(HKEY_CURRENT_USER, r"Control Panel\Mouse", "MouseSpeed").as_deref() == Some("0")
        }
        TweakId::EnableGameMode => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\GameBar",
                "AutoGameModeEnabled",
            ) == Some(1)
        }
        TweakId::DisableNetworkThrottling => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile",
                "NetworkThrottlingIndex",
            ) == Some(0xFFFF_FFFF)
        }
        TweakId::ShowFileExtensions => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                "HideFileExt",
            ) == Some(0)
        }
        TweakId::ShowHiddenFiles => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                "Hidden",
            ) == Some(1)
        }
        TweakId::LaunchExplorerToThisPc => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                "LaunchTo",
            ) == Some(1)
        }
        TweakId::ClassicContextMenu => RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags(
                r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32",
                KEY_READ,
            )
            .is_ok(),
        TweakId::EnableTaskbarEndTask => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDeveloperSettings",
                "TaskbarEndTask",
            ) == Some(1)
        }
        TweakId::AlignTaskbarLeft => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                "TaskbarAl",
            ) == Some(0)
        }
        TweakId::HideCopilotAndWidgets => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                "ShowCopilotButton",
            ) == Some(0)
                && read_u32(
                    HKEY_CURRENT_USER,
                    r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                    "TaskbarDa",
                ) == Some(0)
        }
        TweakId::DisableStickyKeysPopup => {
            read_str(
                HKEY_CURRENT_USER,
                r"Control Panel\Accessibility\StickyKeys",
                "Flags",
            )
            .as_deref()
                == Some("506")
        }
        TweakId::DisableFastStartup => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SYSTEM\CurrentControlSet\Control\Session Manager\Power",
                "HiberbootEnabled",
            ) == Some(0)
        }
        TweakId::DisableWerReporting => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\Microsoft\Windows\Windows Error Reporting",
                "Disabled",
            ) == Some(1)
        }
        TweakId::DisableAutoRebootUpdates => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU",
                "NoAutoRebootWithLoggedOnUsers",
            ) == Some(1)
        }
        TweakId::EnableStorageSense => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\StorageSense\Parameters\StoragePolicy",
                "01",
            ) == Some(1)
        }
        TweakId::DisableQuickAccessRecent => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Explorer",
                "ShowRecent",
            ) == Some(0)
        }
        TweakId::DisableSearchHighlights => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\SearchSettings",
                "IsDynamicSearchBoxEnabled",
            ) == Some(0)
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
        // ── PRIVACY ──────────────────────────────────────────────────────────
        (
            TweakId::DisableTelemetry,
            TweakCategory::Privacy,
            "Minimal Diagnostic Telemetry",
            "Telemetria Diagnóstica Mínima",
            "Sets Windows DataCollection policy to minimum security level (0)",
            "Define a coleta diagnóstica do Windows no nível mínimo (0)",
            true,
            true,
        ),
        (
            TweakId::DisableAdvertisingId,
            TweakCategory::Privacy,
            "Disable Advertising ID",
            "Desativar ID de Publicidade",
            "Stops apps from profiling across sessions using Windows Ad ID",
            "Impede aplicativos de rastrearem seu perfil via ID de anúncio",
            true,
            false,
        ),
        (
            TweakId::DisableStartBingSearch,
            TweakCategory::Privacy,
            "Local-Only Start Menu Search",
            "Busca 100% Local no Menu Iniciar",
            "Disables Bing web results so Start Menu searches respond instantaneously",
            "Remove resultados da web Bing e deixa a busca do Iniciar instantânea",
            true,
            false,
        ),
        (
            TweakId::DisableTipsAndAds,
            TweakCategory::Privacy,
            "Disable Windows Tips & Suggested Ads",
            "Desativar Dicas e Anúncios do Windows",
            "Removes promotional app suggestions from Start, Settings, and Lock Screen",
            "Remove sugestões promocionais no Iniciar, Configurações e tela de bloqueio",
            true,
            false,
        ),
        (
            TweakId::DisableActivityHistory,
            TweakCategory::Privacy,
            "Disable Activity History Collection",
            "Desativar Histórico de Atividades",
            "Stops Windows from logging and syncing local app timeline activity",
            "Impede o Windows de registrar e sincronizar histórico de uso",
            true,
            true,
        ),
        (
            TweakId::DisableTailoredExperiences,
            TweakCategory::Privacy,
            "Disable Tailored Experiences & Feedback",
            "Desativar Experiências Personalizadas",
            "Prevents diagnostic data from being used for personalized prompts and surveys",
            "Evita recomendações personalizadas e pedidos de feedback do Windows",
            true,
            false,
        ),
        // ── PERFORMANCE ──────────────────────────────────────────────────────
        (
            TweakId::HighPerformancePowerPlan,
            TweakCategory::Performance,
            "High / Ultimate Performance Power Plan",
            "Plano de Energia Alto Desempenho",
            "Activates High or Ultimate Performance scheme to prevent aggressive CPU downclocking",
            "Ativa Alto Desempenho ou Desempenho Máximo evitando queda de clock",
            true,
            false,
        ),
        (
            TweakId::DisableBackgroundApps,
            TweakCategory::Performance,
            "Disable Background UWP Apps",
            "Desativar Apps UWP em Segundo Plano",
            "Prevents unused Microsoft Store apps from waking up in the background",
            "Impede aplicativos da Microsoft Store de rodarem fechados em segundo plano",
            true,
            false,
        ),
        (
            TweakId::DisableStartupDelay,
            TweakCategory::Performance,
            "Remove Boot Startup Delay",
            "Remover Atraso de Inicialização",
            "Sets Explorer Serialize StartupDelayInMSec to 0 for faster desktop readiness",
            "Define StartupDelayInMSec como 0 para apps carregarem mais rápido no boot",
            true,
            false,
        ),
        (
            TweakId::DisableVisualTransparency,
            TweakCategory::Performance,
            "Disable Window Transparency Effects",
            "Desativar Efeitos de Transparência",
            "Reduces DWM GPU compositor load on laptops and integrated graphics",
            "Reduz carga do compositor DWM em notebooks e GPUs integradas",
            false,
            false,
        ),
        (
            TweakId::DisableDeliveryOptP2p,
            TweakCategory::Performance,
            "Disable Windows Update P2P Uploads",
            "Desativar Upload P2P do Windows Update",
            "Stops Windows from uploading update chunks to other PCs over your connection",
            "Impede o Windows de usar sua banda de upload para enviar updates a outros PCs",
            true,
            true,
        ),
        (
            TweakId::DisableHibernation,
            TweakCategory::Performance,
            "Disable Hibernation (Free hiberfil.sys)",
            "Desativar Hibernação (Liberar hiberfil.sys)",
            "Reclaims several gigabytes on C: (note: disables Windows Fast Startup)",
            "Libera vários GB no SSD (obs: desativa a Inicialização Rápida)",
            false,
            true,
        ),
        // ── GAMING ───────────────────────────────────────────────────────────
        (
            TweakId::DisableGameDvr,
            TweakCategory::Gaming,
            "Disable Game DVR Background Capture",
            "Desativar Gravação Game DVR",
            "Eliminates background Xbox capture overhead and frame-time spikes",
            "Elimina overhead de captura em segundo plano e micro-stutters em jogos",
            true,
            false,
        ),
        (
            TweakId::DisableMouseAccel,
            TweakCategory::Gaming,
            "Raw 1:1 Mouse Input (Disable Acceleration)",
            "Mouse 1:1 (Desativar Aceleração)",
            "Disables 'Enhance pointer precision' for consistent cursor and aim movement",
            "Desliga 'Aprimorar precisão do ponteiro' para resposta linear 1:1",
            true,
            false,
        ),
        (
            TweakId::EnableGameMode,
            TweakCategory::Gaming,
            "Enable Windows Game Mode",
            "Ativar Modo de Jogo do Windows",
            "Pauses background updates and prioritizes foreground game threads",
            "Pausa atualizações em segundo plano e prioriza o jogo aberto",
            true,
            false,
        ),
        (
            TweakId::DisableNetworkThrottling,
            TweakCategory::Gaming,
            "Disable Multimedia Network Throttling",
            "Desativar Throttling de Rede Multimídia",
            "Sets NetworkThrottlingIndex to 0xFFFFFFFF to prevent packet pacing jitter",
            "Define NetworkThrottlingIndex como 0xFFFFFFFF reduzindo jitter em jogos",
            false,
            true,
        ),
        // ── EXPLORER & INTERFACE ─────────────────────────────────────────────
        (
            TweakId::ShowFileExtensions,
            TweakCategory::Explorer,
            "Show File Name Extensions",
            "Mostrar Extensões de Arquivos",
            "Always displays .exe, .zip, .json, .dll extensions in File Explorer",
            "Exibe sempre extensões .exe, .zip, .json no Explorador de Arquivos",
            true,
            false,
        ),
        (
            TweakId::ShowHiddenFiles,
            TweakCategory::Explorer,
            "Show Hidden Files & Folders",
            "Mostrar Arquivos e Pastas Ocultas",
            "Makes AppData and hidden directories immediately browsable",
            "Torna pastas como AppData imediatamente visíveis no Explorador",
            true,
            false,
        ),
        (
            TweakId::LaunchExplorerToThisPc,
            TweakCategory::Explorer,
            "Open File Explorer to 'This PC'",
            "Abrir Explorador em 'Este Computador'",
            "Opens directly to drives and folders instead of Home / Quick Access",
            "Abre diretamente nos discos e partições em vez da tela Início",
            true,
            false,
        ),
        (
            TweakId::ClassicContextMenu,
            TweakCategory::Explorer,
            "Classic Right-Click Context Menu (Win11)",
            "Menu de Contexto Clássico (Win11)",
            "Restores the full right-click menu immediately without 'Show more options'",
            "Restaura o menu completo ao clicar com botão direito no Windows 11",
            true,
            false,
        ),
        (
            TweakId::EnableTaskbarEndTask,
            TweakCategory::Explorer,
            "Enable 'End Task' on Taskbar (Win11)",
            "Ativar 'Finalizar Tarefa' na Barra (Win11)",
            "Adds instant End Task directly to taskbar right-click menus",
            "Adiciona opção rápida de Finalizar Tarefa no menu da barra de tarefas",
            true,
            false,
        ),
        (
            TweakId::AlignTaskbarLeft,
            TweakCategory::Explorer,
            "Align Taskbar to Left (Win11)",
            "Alinhar Barra de Tarefas à Esquerda",
            "Anchors Start button and pinned icons to the classic bottom-left corner",
            "Posiciona o Menu Iniciar e ícones fixados no canto esquerdo clássico",
            false,
            false,
        ),
        (
            TweakId::HideCopilotAndWidgets,
            TweakCategory::Explorer,
            "Hide Copilot & Widgets from Taskbar",
            "Ocultar Copilot e Widgets da Barra",
            "Declutters taskbar by hiding both Copilot and Widgets news buttons",
            "Limpa a barra de tarefas ocultando os botões de Copilot e Widgets",
            true,
            false,
        ),
        // ── POWER & SYSTEM ───────────────────────────────────────────────────
        (
            TweakId::DisableStickyKeysPopup,
            TweakCategory::PowerAndSystem,
            "Disable Sticky Keys Shift Popup",
            "Desativar Aviso de Teclas de Aderência",
            "Prevents pressing Shift 5 times from minimizing fullscreen apps",
            "Evita a interrupção em tela cheia ao pressionar Shift 5 vezes",
            true,
            false,
        ),
        (
            TweakId::DisableFastStartup,
            TweakCategory::PowerAndSystem,
            "Disable Fast Startup (Clean Cold Boots)",
            "Desativar Inicialização Rápida",
            "Ensures Shutdown performs a full kernel reset for cleaner dual-boot & driver stability",
            "Garante desligamento real do kernel para estabilidade de drivers e dual-boot",
            false,
            true,
        ),
        (
            TweakId::DisableWerReporting,
            TweakCategory::PowerAndSystem,
            "Disable Windows Error Reporting",
            "Desativar Relatório de Erros (WER)",
            "Stops Windows from generating and uploading background crash dump queues",
            "Impede o acúmulo e envio automático de relatórios de falha em segundo plano",
            false,
            true,
        ),
        (
            TweakId::DisableAutoRebootUpdates,
            TweakCategory::PowerAndSystem,
            "Prevent Forced Update Reboots While Logged In",
            "Evitar Reinício Forçado do Windows Update",
            "Blocks Windows Update from automatically restarting while you have an active session",
            "Impede o Windows Update de reiniciar o PC automaticamente durante sua sessão",
            true,
            true,
        ),
        // ── MAINTENANCE ──────────────────────────────────────────────────────
        (
            TweakId::EnableStorageSense,
            TweakCategory::Maintenance,
            "Enable Automatic Storage Sense",
            "Ativar Sensor de Armazenamento Automático",
            "Allows Windows to periodically clean expired temp files automatically",
            "Permite que o Windows limpe arquivos temporários antigos periodicamente",
            true,
            false,
        ),
        (
            TweakId::DisableQuickAccessRecent,
            TweakCategory::Maintenance,
            "Hide Recently Used Files in Explorer",
            "Ocultar Arquivos Recentes no Explorador",
            "Keeps Explorer Home clean by not tracking recently opened documents",
            "Mantém o Explorador limpo sem listar documentos abertos recentemente",
            false,
            false,
        ),
        (
            TweakId::DisableSearchHighlights,
            TweakCategory::Maintenance,
            "Disable Taskbar Search Highlights",
            "Desativar Destaques na Caixa de Pesquisa",
            "Removes daily illustrations and trending banners inside Windows Search",
            "Remove ilustrações diárias e banners de tendências na busca do Windows",
            true,
            false,
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

/// Applies (`enable = true`) or cleanly reverts (`enable = false`) a tweak, then verifies
/// the resulting system state instead of blindly reporting success.
pub fn set_tweak_verified(id: TweakId, enable: bool) -> Result<bool, String> {
    apply_tweak_raw(id, enable)?;
    let actual = detect_tweak_state(id);
    if actual != enable {
        return Err("Setting was written, but Windows did not reflect the new state (Administrator privileges or restart may be required).".into());
    }
    Ok(actual)
}

fn apply_tweak_raw(id: TweakId, enable: bool) -> Result<(), String> {
    match id {
        TweakId::DisableTelemetry => write_u32(
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Policies\Microsoft\Windows\DataCollection",
            "AllowTelemetry",
            if enable { 0 } else { 1 },
        ),
        TweakId::DisableAdvertisingId => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo",
            "Enabled",
            if enable { 0 } else { 1 },
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
        TweakId::DisableTipsAndAds => {
            let v = if enable { 0 } else { 1 };
            let path = r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager";
            write_u32(HKEY_CURRENT_USER, path, "SubscribedContent-338389Enabled", v)?;
            let _ = write_u32(HKEY_CURRENT_USER, path, "SubscribedContent-338388Enabled", v);
            let _ = write_u32(HKEY_CURRENT_USER, path, "SubscribedContent-310093Enabled", v);
            let _ = write_u32(HKEY_CURRENT_USER, path, "SystemPaneSuggestionsEnabled", v);
            let _ = write_u32(HKEY_CURRENT_USER, path, "SoftLandingEnabled", v);
            Ok(())
        }
        TweakId::DisableActivityHistory => {
            let v = if enable { 0 } else { 1 };
            let path = r"SOFTWARE\Policies\Microsoft\Windows\System";
            write_u32(HKEY_LOCAL_MACHINE, path, "EnableActivityFeed", v)?;
            let _ = write_u32(HKEY_LOCAL_MACHINE, path, "PublishUserActivities", v);
            let _ = write_u32(HKEY_LOCAL_MACHINE, path, "UploadUserActivities", v);
            Ok(())
        }
        TweakId::DisableTailoredExperiences => {
            let v = if enable { 0 } else { 1 };
            write_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Privacy",
                "TailoredExperiencesWithDiagnosticDataEnabled",
                v,
            )?;
            let _ = write_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Siuf\Rules",
                "NumberOfSIUFInPeriod",
                if enable { 0 } else { 1 },
            );
            Ok(())
        }
        TweakId::HighPerformancePowerPlan => {
            if enable {
                let _ = hidden_std_cmd("powercfg")
                    .args(["-duplicatescheme", "e9a42b02-d5df-448d-aa00-03f14749eb61"])
                    .output();
                let high = hidden_std_cmd("powercfg")
                    .args(["/setactive", "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c"])
                    .output()
                    .map_err(|e| e.to_string())?;
                if !high.status.success() {
                    let _ = hidden_std_cmd("powercfg")
                        .args(["/setactive", "e9a42b02-d5df-448d-aa00-03f14749eb61"])
                        .output();
                }
            } else {
                let _ = hidden_std_cmd("powercfg")
                    .args(["/setactive", "381b4222-f694-41f0-9685-ff5bb260df2e"])
                    .output();
            }
            Ok(())
        }
        TweakId::DisableBackgroundApps => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications",
            "GlobalUserDisabled",
            if enable { 1 } else { 0 },
        ),
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
        TweakId::DisableVisualTransparency => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
            "EnableTransparency",
            if enable { 0 } else { 1 },
        ),
        TweakId::DisableDeliveryOptP2p => write_u32(
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization",
            "DODownloadMode",
            if enable { 0 } else { 1 },
        ),
        TweakId::DisableHibernation => {
            let arg = if enable { "off" } else { "on" };
            let out = hidden_std_cmd("powercfg")
                .args(["/h", arg])
                .output()
                .map_err(|e| e.to_string())?;
            if out.status.success() {
                Ok(())
            } else {
                Err("Administrator privileges are required to toggle Hibernation.".into())
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
        TweakId::DisableMouseAccel => {
            let path = r"Control Panel\Mouse";
            if enable {
                write_str(HKEY_CURRENT_USER, path, "MouseSpeed", "0")?;
                write_str(HKEY_CURRENT_USER, path, "MouseThreshold1", "0")?;
                write_str(HKEY_CURRENT_USER, path, "MouseThreshold2", "0")
            } else {
                write_str(HKEY_CURRENT_USER, path, "MouseSpeed", "1")?;
                write_str(HKEY_CURRENT_USER, path, "MouseThreshold1", "6")?;
                write_str(HKEY_CURRENT_USER, path, "MouseThreshold2", "10")
            }
        }
        TweakId::EnableGameMode => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\GameBar",
            "AutoGameModeEnabled",
            if enable { 1 } else { 0 },
        ),
        TweakId::DisableNetworkThrottling => write_u32(
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile",
            "NetworkThrottlingIndex",
            if enable { 0xFFFF_FFFF } else { 10 },
        ),
        TweakId::ShowFileExtensions => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
            "HideFileExt",
            if enable { 0 } else { 1 },
        ),
        TweakId::ShowHiddenFiles => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
            "Hidden",
            if enable { 1 } else { 2 },
        ),
        TweakId::LaunchExplorerToThisPc => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
            "LaunchTo",
            if enable { 1 } else { 2 },
        ),
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
        TweakId::EnableTaskbarEndTask => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDeveloperSettings",
            "TaskbarEndTask",
            if enable { 1 } else { 0 },
        ),
        TweakId::AlignTaskbarLeft => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
            "TaskbarAl",
            if enable { 0 } else { 1 },
        ),
        TweakId::HideCopilotAndWidgets => {
            let v = if enable { 0 } else { 1 };
            let path = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
            write_u32(HKEY_CURRENT_USER, path, "ShowCopilotButton", v)?;
            write_u32(HKEY_CURRENT_USER, path, "TaskbarDa", v)
        }
        TweakId::DisableStickyKeysPopup => write_str(
            HKEY_CURRENT_USER,
            r"Control Panel\Accessibility\StickyKeys",
            "Flags",
            if enable { "506" } else { "510" },
        ),
        TweakId::DisableFastStartup => write_u32(
            HKEY_LOCAL_MACHINE,
            r"SYSTEM\CurrentControlSet\Control\Session Manager\Power",
            "HiberbootEnabled",
            if enable { 0 } else { 1 },
        ),
        TweakId::DisableWerReporting => write_u32(
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows\Windows Error Reporting",
            "Disabled",
            if enable { 1 } else { 0 },
        ),
        TweakId::DisableAutoRebootUpdates => write_u32(
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU",
            "NoAutoRebootWithLoggedOnUsers",
            if enable { 1 } else { 0 },
        ),
        TweakId::EnableStorageSense => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\StorageSense\Parameters\StoragePolicy",
            "01",
            if enable { 1 } else { 0 },
        ),
        TweakId::DisableQuickAccessRecent => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Explorer",
            "ShowRecent",
            if enable { 0 } else { 1 },
        ),
        TweakId::DisableSearchHighlights => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\SearchSettings",
            "IsDynamicSearchBoxEnabled",
            if enable { 0 } else { 1 },
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_all_30_tweaks_across_all_categories() {
        let all = load_all_tweaks();
        assert_eq!(all.len(), 30);
        for &cat in TweakCategory::ALL {
            assert!(
                all.iter().any(|t| t.category == cat),
                "Missing tweaks in category {:?}",
                cat
            );
        }
    }
}

