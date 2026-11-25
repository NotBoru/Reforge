use crate::core::winapi::hidden_std_cmd;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TweakCategory {
    Privacy,
    Performance,
    Interface,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TweakId {
    // Privacy
    DisableTelemetry,
    DisableAdvertisingId,
    DisableStartBingSearch,
    DisableTipsAndAds,
    DisableActivityHistory,
    DisableCopilotTaskbar,
    // Performance
    HighPerformancePowerPlan,
    DisableGameDvr,
    DisableMouseAccel,
    DisableStartupDelay,
    DisableBackgroundApps,
    DisableHibernation,
    // Interface & UX
    ShowFileExtensions,
    ShowHiddenFiles,
    ClassicContextMenu,
    AlignTaskbarLeft,
    DisableStickyKeysPopup,
    EnableTaskbarEndTask,
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
        .map_err(|e| format!("Registry write failed (Admin may be required): {e}"))?;
    key.set_value(name, &val).map_err(|e| e.to_string())
}

fn write_str(root: winreg::HKEY, path: &str, name: &str, val: &str) -> Result<(), String> {
    let (key, _) = RegKey::predef(root)
        .create_subkey(path)
        .map_err(|e| format!("Registry write failed (Admin may be required): {e}"))?;
    key.set_value(name, &val).map_err(|e| e.to_string())
}

fn detect_tweak_state(id: TweakId) -> bool {
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
        TweakId::DisableCopilotTaskbar => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                "ShowCopilotButton",
            ) == Some(0)
        }
        TweakId::HighPerformancePowerPlan => {
            hidden_std_cmd("powercfg")
                .arg("/getactivescheme")
                .output()
                .map(|o| {
                    let s = String::from_utf8_lossy(&o.stdout).to_lowercase();
                    s.contains("8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c")
                        || s.contains("e9a42b02-d5df-448d-aa00-03f14749eb61")
                })
                .unwrap_or(false)
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
        TweakId::DisableStartupDelay => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\Serialize",
                "StartupDelayInMSec",
            ) == Some(0)
        }
        TweakId::DisableBackgroundApps => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications",
                "GlobalUserDisabled",
            ) == Some(1)
        }
        TweakId::DisableHibernation => {
            read_u32(
                HKEY_LOCAL_MACHINE,
                r"SYSTEM\CurrentControlSet\Control\Power",
                "HibernateEnabled",
            ) == Some(0)
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
        TweakId::ClassicContextMenu => RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags(
                r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32",
                KEY_READ,
            )
            .is_ok(),
        TweakId::AlignTaskbarLeft => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                "TaskbarAl",
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
        TweakId::EnableTaskbarEndTask => {
            read_u32(
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDeveloperSettings",
                "TaskbarEndTask",
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
        (
            TweakId::DisableTelemetry,
            TweakCategory::Privacy,
            "Disable Diagnostic Telemetry",
            "Desativar Telemetria de Diagnóstico",
            "Sets Windows DataCollection policy to minimum (0)",
            "Define a coleta de dados diagnósticos do Windows no mínimo (0)",
            true,
            true,
        ),
        (
            TweakId::DisableAdvertisingId,
            TweakCategory::Privacy,
            "Disable Advertising ID",
            "Desativar ID de Publicidade",
            "Prevents apps from tracking you with a unique Windows ad ID",
            "Impede que aplicativos usem o ID de anúncio do Windows",
            true,
            false,
        ),
        (
            TweakId::DisableStartBingSearch,
            TweakCategory::Privacy,
            "Disable Bing Web Search in Start",
            "Desativar Busca Bing no Menu Iniciar",
            "Makes Start Menu search instant and strictly local",
            "Deixa a busca do Menu Iniciar instantânea e 100% local",
            true,
            false,
        ),
        (
            TweakId::DisableTipsAndAds,
            TweakCategory::Privacy,
            "Disable Start Tips & Suggestions",
            "Desativar Dicas e Anúncios do Windows",
            "Turns off Lock Screen ads, tips, and suggested app promotions",
            "Remove anúncios na tela de bloqueio e sugestões no Iniciar",
            true,
            false,
        ),
        (
            TweakId::DisableActivityHistory,
            TweakCategory::Privacy,
            "Disable Activity History",
            "Desativar Histórico de Atividades",
            "Stops Windows from logging & uploading local app activity",
            "Impede o Windows de registrar e enviar histórico de uso",
            true,
            true,
        ),
        (
            TweakId::DisableCopilotTaskbar,
            TweakCategory::Privacy,
            "Hide Copilot from Taskbar",
            "Ocultar Copilot da Barra de Tarefas",
            "Removes the Copilot button from the Windows 11 taskbar",
            "Remove o botão do Copilot da barra de tarefas",
            true,
            false,
        ),
        (
            TweakId::HighPerformancePowerPlan,
            TweakCategory::Performance,
            "High / Ultimate Performance Power Plan",
            "Plano de Energia Alto Desempenho",
            "Activates Ultimate or High Performance power scheme for zero CPU throttling",
            "Ativa o plano Desempenho Máximo / Alto Desempenho sem throttling",
            true,
            false,
        ),
        (
            TweakId::DisableGameDvr,
            TweakCategory::Performance,
            "Disable Game DVR Background Capture",
            "Desativar Gravação em Segundo Plano (Game DVR)",
            "Eliminates background GPU capture overhead & micro-stutter in games",
            "Elimina overhead de gravação da GPU e micro-stutters em jogos",
            true,
            false,
        ),
        (
            TweakId::DisableMouseAccel,
            TweakCategory::Performance,
            "Disable Mouse Acceleration (Raw 1:1)",
            "Desativar Aceleração do Mouse (1:1)",
            "Turns off 'Enhance pointer precision' for consistent aim & cursor feel",
            "Desliga 'Aprimorar precisão do ponteiro' para resposta 1:1",
            true,
            false,
        ),
        (
            TweakId::DisableStartupDelay,
            TweakCategory::Performance,
            "Remove Windows Startup Delay",
            "Remover Atraso de Inicialização",
            "Sets Explorer Serialize StartupDelayInMSec to 0 for faster boot",
            "Define StartupDelayInMSec como 0 para apps iniciarem mais rápido",
            true,
            false,
        ),
        (
            TweakId::DisableBackgroundApps,
            TweakCategory::Performance,
            "Disable Background UWP Apps",
            "Desativar Apps UWP em Segundo Plano",
            "Stops Store apps from consuming RAM & CPU while closed",
            "Impede apps da Microsoft Store de rodar em segundo plano",
            true,
            false,
        ),
        (
            TweakId::DisableHibernation,
            TweakCategory::Performance,
            "Disable Hibernation (Free hiberfil.sys)",
            "Desativar Hibernação (Liberar hiberfil.sys)",
            "Reclaims several GB of SSD space (note: disables Fast Startup)",
            "Libera vários GB no SSD (obs: desativa Inicialização Rápida)",
            false,
            true,
        ),
        (
            TweakId::ShowFileExtensions,
            TweakCategory::Interface,
            "Show File Name Extensions",
            "Mostrar Extensões de Arquivos",
            "Displays .exe, .json, .rs, .zip extensions in File Explorer",
            "Exibe extensões .exe, .zip, .txt no Explorador de Arquivos",
            true,
            false,
        ),
        (
            TweakId::ShowHiddenFiles,
            TweakCategory::Interface,
            "Show Hidden Files & Folders",
            "Mostrar Arquivos e Pastas Ocultas",
            "Makes AppData and hidden directories visible in Explorer",
            "Torna pastas como AppData visíveis no Explorador de Arquivos",
            true,
            false,
        ),
        (
            TweakId::ClassicContextMenu,
            TweakCategory::Interface,
            "Classic Right-Click Context Menu (Win11)",
            "Menu de Contexto Clássico (Win11)",
            "Restores the full instant right-click menu without 'Show more options'",
            "Restaura o menu completo ao clicar com botão direito no Win11",
            true,
            false,
        ),
        (
            TweakId::AlignTaskbarLeft,
            TweakCategory::Interface,
            "Align Taskbar Left (Win11)",
            "Alinhar Barra de Tarefas à Esquerda",
            "Places the Start button in the classic bottom-left corner",
            "Posiciona o Menu Iniciar no canto inferior esquerdo clássico",
            false,
            false,
        ),
        (
            TweakId::DisableStickyKeysPopup,
            TweakCategory::Interface,
            "Disable Sticky Keys Shift Popup",
            "Desativar Popup de Teclas de Aderência",
            "Prevents pressing Shift 5 times from interrupting fullscreen apps",
            "Evita o aviso irritante ao pressionar Shift 5 vezes",
            true,
            false,
        ),
        (
            TweakId::EnableTaskbarEndTask,
            TweakCategory::Interface,
            "Enable 'End Task' on Taskbar (Win11)",
            "Ativar 'Finalizar Tarefa' na Barra (Win11)",
            "Adds instant End Task directly to taskbar right-click menus",
            "Adiciona opção direta de Finalizar Tarefa na barra de tarefas",
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

/// Applies (`enable = true`) or cleanly reverts (`enable = false`) a system tweak.
pub fn set_tweak(id: TweakId, enable: bool) -> Result<(), String> {
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
        TweakId::DisableCopilotTaskbar => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
            "ShowCopilotButton",
            if enable { 0 } else { 1 },
        ),
        TweakId::HighPerformancePowerPlan => {
            if enable {
                // Try Ultimate Performance first, fallback to High Performance
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
                // Restore Balanced power plan
                let _ = hidden_std_cmd("powercfg")
                    .args(["/setactive", "381b4222-f694-41f0-9685-ff5bb260df2e"])
                    .output();
            }
            Ok(())
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
        TweakId::DisableBackgroundApps => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications",
            "GlobalUserDisabled",
            if enable { 1 } else { 0 },
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
        TweakId::AlignTaskbarLeft => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
            "TaskbarAl",
            if enable { 0 } else { 1 },
        ),
        TweakId::DisableStickyKeysPopup => write_str(
            HKEY_CURRENT_USER,
            r"Control Panel\Accessibility\StickyKeys",
            "Flags",
            if enable { "506" } else { "510" },
        ),
        TweakId::EnableTaskbarEndTask => write_u32(
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDeveloperSettings",
            "TaskbarEndTask",
            if enable { 1 } else { 0 },
        ),
    }
}
