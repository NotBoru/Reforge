use crate::core::winapi::hidden_tokio_cmd;
use serde::Deserialize;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BloatSafety {
    Safe,
    Moderate,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BloatPackage {
    pub package_name: String,
    pub package_full_name: String,
    pub display_name: String,
    pub desc_en: String,
    pub desc_pt: String,
    pub category: &'static str,
    pub safety: BloatSafety,
    pub selected: bool,
    pub version: String,
    pub is_known_bloat: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeepDebloatAction {
    RemoveOneDrive,
    RemoveWidgetsPack,
    DisableRecallAndCopilot,
    BlockAutoInstalledBloat,
    RemoveLegacyCapabilities,
    CleanTelemetryTasks,
}

#[derive(Debug, Clone, Copy)]
pub struct DeepDebloatTool {
    pub action: DeepDebloatAction,
    pub name_en: &'static str,
    pub name_pt: &'static str,
    pub desc_en: &'static str,
    pub desc_pt: &'static str,
}

pub const DEEP_DEBLOAT_TOOLS: &[DeepDebloatTool] = &[
    DeepDebloatTool {
        action: DeepDebloatAction::RemoveOneDrive,
        name_en: "Uninstall Microsoft OneDrive",
        name_pt: "Desinstalar Microsoft OneDrive",
        desc_en: "Kills OneDrive, runs official uninstaller & hides Explorer sidebar pin",
        desc_pt: "Encerra o OneDrive, roda o desinstalador oficial e remove do Explorador",
    },
    DeepDebloatTool {
        action: DeepDebloatAction::RemoveWidgetsPack,
        name_en: "Remove Windows Widgets & Web Experience",
        name_pt: "Remover Widgets e Web Experience Pack",
        desc_en: "Removes background WebExperience widget host & disables taskbar news feed",
        desc_pt: "Remove o host WebExperience em segundo plano e o feed de notícias",
    },
    DeepDebloatTool {
        action: DeepDebloatAction::DisableRecallAndCopilot,
        name_en: "Disable Windows Recall & Copilot AI",
        name_pt: "Desativar Windows Recall e Copilot AI",
        desc_en: "Disables AI snapshot indexing (Recall), Copilot policies & taskbar button",
        desc_pt: "Desativa capturas de IA (Recall), políticas do Copilot e botão da barra",
    },
    DeepDebloatTool {
        action: DeepDebloatAction::BlockAutoInstalledBloat,
        name_en: "Block Silent Bloatware Re-Installation",
        name_pt: "Bloquear Reinstalação Automática de Bloatware",
        desc_en: "Stops Windows from silently downloading Candy Crush, TikTok & OEM apps",
        desc_pt: "Impede o Windows de baixar automaticamente TikTok, jogos e apps OEM",
    },
    DeepDebloatTool {
        action: DeepDebloatAction::RemoveLegacyCapabilities,
        name_en: "Remove Legacy Optional Features",
        name_pt: "Remover Recursos Opcionais Legados",
        desc_en: "Removes Steps Recorder, Math Recognizer, Fax & Scan, Quick Assist & IE Mode",
        desc_pt: "Remove Gravador de Passos, Math Recognizer, Fax, Assistência Rápida e IE",
    },
    DeepDebloatTool {
        action: DeepDebloatAction::CleanTelemetryTasks,
        name_en: "Disable Telemetry & CEIP Scheduled Tasks",
        name_pt: "Desativar Tarefas Agendadas de Telemetria",
        desc_en: "Disables Compatibility Appraiser, CEIP Consolidator & DiskDiagnostic tasks",
        desc_pt: "Desativa tarefas Compatibility Appraiser, CEIP e diagnósticos em background",
    },
];

struct KnownBloat {
    prefix: &'static str,
    display: &'static str,
    desc_en: &'static str,
    desc_pt: &'static str,
    category: &'static str,
    safety: BloatSafety,
}

macro_rules! kb {
    ($pref:expr, $disp:expr, $en:expr, $pt:expr, $cat:expr, $safe:expr) => {
        KnownBloat {
            prefix: $pref,
            display: $disp,
            desc_en: $en,
            desc_pt: $pt,
            category: $cat,
            safety: $safe,
        }
    };
}

const KNOWN_BLOAT: &[KnownBloat] = &[
    // Microsoft & AI Bloat
    kb!("Clipchamp.Clipchamp", "Microsoft Clipchamp", "Pre-installed cloud video editor", "Editor de vídeo na nuvem pré-instalado", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.Copilot", "Microsoft Copilot", "Bundled Copilot web wrapper", "App pré-instalado do Copilot", "AI & Web", BloatSafety::Safe),
    kb!("Microsoft.549981C3F5F10", "Cortana", "Legacy deprecated voice assistant", "Assistente de voz descontinuada", "AI & Web", BloatSafety::Safe),
    kb!("MicrosoftWindows.Client.WebExperience", "Windows Widgets Pack", "Background Edge WebView news & widgets host", "Host WebView de Widgets e notícias em segundo plano", "AI & Web", BloatSafety::Safe),
    kb!("MicrosoftWindows.CrossDevice", "Cross Device Experience", "Background phone/cross-device host", "Serviço de integração entre dispositivos", "Microsoft", BloatSafety::Moderate),
    kb!("Microsoft.WindowsFeedbackHub", "Feedback Hub", "Telemetry & diagnostic feedback app", "App de envio de feedback e diagnóstico", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.GetHelp", "Get Help", "Microsoft support chat wrapper", "Atalho de suporte da Microsoft", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.Getstarted", "Microsoft Tips", "Windows onboarding tips & prompts", "Dicas e notificações iniciais", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.MicrosoftOfficeHub", "Microsoft 365 Hub", "Promotional Microsoft 365 launcher stub", "Atalho promocional do Microsoft 365", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.Office.OneNote", "OneNote (Store)", "Pre-installed UWP OneNote app", "App UWP pré-instalado do OneNote", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.People", "Microsoft People", "Unused contacts integration app", "App de contatos legado", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.Todos", "Microsoft To Do", "Bundled task list client", "Lista de tarefas da Microsoft", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.PowerAutomateDesktop", "Power Automate Desktop", "Enterprise workflow stub", "Automação corporativa pré-instalada", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.YourPhone", "Phone Link", "Background Android/iOS syncing app", "Sincronização em segundo plano com celular", "Microsoft", BloatSafety::Moderate),
    kb!("Microsoft.MicrosoftStickyNotes", "Sticky Notes", "Desktop sticky notes UWP app", "Notas autoadesivas do Windows", "Microsoft", BloatSafety::Moderate),
    kb!("Microsoft.WindowsMaps", "Windows Maps", "Pre-installed offline maps app", "Aplicativo de mapas da Microsoft", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.WindowsAlarms", "Windows Clock & Alarms", "Bundled alarm/timer UWP app", "Relógio e alarmes UWP", "Microsoft", BloatSafety::Moderate),
    kb!("Microsoft.WindowsCamera", "Windows Camera", "Built-in webcam UWP app", "Aplicativo de câmera nativo", "Microsoft", BloatSafety::Moderate),
    kb!("Microsoft.WindowsSoundRecorder", "Sound Recorder", "Basic Windows voice recorder", "Gravador de voz básico", "Microsoft", BloatSafety::Moderate),
    kb!("MicrosoftCorporationII.QuickAssist", "Quick Assist", "Remote assistance support tool", "Assistência remota da Microsoft", "Microsoft", BloatSafety::Safe),
    kb!("MicrosoftCorporationII.MicrosoftFamily", "Family Safety", "Parental controls stub app", "App de controle parental Microsoft", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.Windows.DevHome", "Dev Home", "Deprecated developer dashboard", "Dashboard Dev Home descontinuado", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.MicrosoftJournal", "Microsoft Journal", "Stylus note-taking app", "App de anotações para caneta", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.Whiteboard", "Microsoft Whiteboard", "Cloud collaborative whiteboard", "Quadro branco online Microsoft", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.MixedReality.Portal", "Mixed Reality Portal", "Deprecated VR headset portal", "Portal VR descontinuado", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.Microsoft3DViewer", "3D Viewer", "Legacy 3D model viewer", "Visualizador 3D legado", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.MSPaint", "Paint 3D", "Legacy Paint 3D UWP app", "Paint 3D legado da Microsoft", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.3DBuilder", "3D Builder", "Legacy 3D printing app", "App legado de impressão 3D", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.Print3D", "Print 3D", "3D printer spooler UWP app", "App UWP de impressão 3D", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.OneConnect", "Mobile Plans", "Cellular SIM plans stub", "Atalho de planos de dados móveis", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.Messaging", "Microsoft Messaging", "Legacy SMS messaging stub", "App legado de SMS", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.Wallet", "Microsoft Pay / Wallet", "Legacy payment wallet app", "Carteira legada Microsoft", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.NetworkSpeedTest", "Network Speed Test", "Bundled speed test app", "Teste de velocidade pré-instalado", "Microsoft", BloatSafety::Safe),
    kb!("Microsoft.Sway", "Microsoft Sway", "Presentation web stub", "Atalho web do Microsoft Sway", "Microsoft", BloatSafety::Safe),
    // Bing & Feeds
    kb!("Microsoft.BingNews", "Microsoft News", "Bing News feed app", "App de notícias Bing", "Bing & Feeds", BloatSafety::Safe),
    kb!("Microsoft.BingWeather", "MSN Weather", "Bing Weather app", "App de clima Bing", "Bing & Feeds", BloatSafety::Safe),
    kb!("Microsoft.BingSearch", "Microsoft Bing Search", "Web search integration package", "Pacote de integração Bing", "Bing & Feeds", BloatSafety::Safe),
    kb!("Microsoft.BingFinance", "MSN Money", "Bing Finance stock tracker", "App de finanças Bing", "Bing & Feeds", BloatSafety::Safe),
    kb!("Microsoft.BingSports", "MSN Sports", "Bing Sports news tracker", "App de esportes Bing", "Bing & Feeds", BloatSafety::Safe),
    kb!("Microsoft.BingTravel", "MSN Travel", "Bing Travel guide stub", "App de viagens Bing", "Bing & Feeds", BloatSafety::Safe),
    kb!("Microsoft.BingFoodAndDrink", "MSN Food & Drink", "Legacy recipe app", "App legado de receitas", "Bing & Feeds", BloatSafety::Safe),
    // Communication
    kb!("MSTeams", "Microsoft Teams (Personal)", "Bundled Win11 consumer Teams", "Microsoft Teams pré-instalado", "Communication", BloatSafety::Safe),
    kb!("MicrosoftTeams", "Microsoft Teams Chat", "Legacy Teams Chat integration", "Integração legada Teams Chat", "Communication", BloatSafety::Safe),
    kb!("Microsoft.SkypeApp", "Skype", "Pre-installed Skype UWP client", "Skype UWP pré-instalado", "Communication", BloatSafety::Safe),
    kb!("Microsoft.OutlookForWindows", "Outlook (New)", "WebView replacement Mail client", "Novo Outlook web pré-instalado", "Communication", BloatSafety::Safe),
    kb!("Microsoft.WindowsCommunicationsApps", "Mail & Calendar", "Legacy Mail & Calendar suite", "Email e Calendário legados", "Communication", BloatSafety::Safe),
    // Media & Xbox
    kb!("Microsoft.ZuneVideo", "Movies & TV", "Legacy Microsoft video player", "Reprodutor Filmes e TV", "Media & Xbox", BloatSafety::Safe),
    kb!("Microsoft.ZuneMusic", "Windows Media Player", "Built-in UWP media player", "Reprodutor UWP do Windows", "Media & Xbox", BloatSafety::Moderate),
    kb!("Microsoft.MicrosoftSolitaireCollection", "Microsoft Solitaire", "Ad-supported bundled card games", "Jogos Paciência com anúncios", "Media & Xbox", BloatSafety::Safe),
    kb!("Microsoft.XboxGamingOverlay", "Xbox Game Bar", "In-game Xbox overlay", "Overlay Xbox Game Bar", "Media & Xbox", BloatSafety::Moderate),
    kb!("Microsoft.Xbox.TCUI", "Xbox Live In-Game", "Xbox Live UI integration", "Interface Xbox Live em jogos", "Media & Xbox", BloatSafety::Moderate),
    kb!("Microsoft.XboxSpeechToTextOverlay", "Xbox Speech To Text", "Xbox speech accessibility overlay", "Overlay de voz Xbox", "Media & Xbox", BloatSafety::Safe),
    kb!("Microsoft.XboxGameOverlay", "Xbox Game Bar Plugin", "Legacy Game Bar overlay plugin", "Plugin legado do Game Bar", "Media & Xbox", BloatSafety::Moderate),
    kb!("Microsoft.XboxApp", "Xbox Companion (Legacy)", "Legacy Xbox Console Companion", "App Xbox Companion antigo", "Media & Xbox", BloatSafety::Safe),
    kb!("Microsoft.GamingApp", "Xbox App", "Xbox Game Pass storefront", "App Xbox / Game Pass", "Media & Xbox", BloatSafety::Moderate),
    // OEM & Sponsored Third-Party Bloat
    kb!("SpotifyAB.SpotifyMusic", "Spotify (Store Preinstall)", "OEM Store Spotify stub", "Versão loja pré-instalada do Spotify", "OEM & Third-Party", BloatSafety::Safe),
    kb!("Disney.37853FC22B2CE", "Disney+ (Preinstall)", "Pre-installed streaming stub", "Atalho pré-instalado Disney+", "OEM & Third-Party", BloatSafety::Safe),
    kb!("BytedancePte.Ltd.TikTok", "TikTok (Preinstall)", "Pre-installed TikTok stub", "Atalho pré-instalado TikTok", "OEM & Third-Party", BloatSafety::Safe),
    kb!("Facebook.InstagramBeta", "Instagram (Preinstall)", "Pre-installed Instagram stub", "Atalho pré-instalado Instagram", "OEM & Third-Party", BloatSafety::Safe),
    kb!("Facebook.Facebook", "Facebook (Preinstall)", "Pre-installed Facebook stub", "Atalho pré-instalado Facebook", "OEM & Third-Party", BloatSafety::Safe),
    kb!("FACEBOOK.317180B0BB486", "Messenger (Preinstall)", "Pre-installed Messenger stub", "Atalho pré-instalado Messenger", "OEM & Third-Party", BloatSafety::Safe),
    kb!("AmazonVideo.PrimeVideo", "Prime Video (Preinstall)", "OEM streaming stub", "Atalho pré-instalado Prime Video", "OEM & Third-Party", BloatSafety::Safe),
    kb!("4DF9E0F8.Netflix", "Netflix (Preinstall)", "OEM streaming stub", "Atalho pré-instalado Netflix", "OEM & Third-Party", BloatSafety::Safe),
    kb!("7EE7776C.LinkedInforWindows", "LinkedIn (Preinstall)", "Pre-installed LinkedIn web wrapper", "Atalho pré-instalado LinkedIn", "OEM & Third-Party", BloatSafety::Safe),
    kb!("king.com.", "King / Candy Crush Game", "Sponsored pre-installed mobile game", "Jogo patrocinado pré-instalado", "OEM & Third-Party", BloatSafety::Safe),
    kb!("5A894077.McAfeeSecurity", "McAfee Security (OEM)", "OEM antivirus bloatware", "Antivírus OEM pré-instalado McAfee", "OEM & Third-Party", BloatSafety::Safe),
    kb!("AD2F1837.", "HP OEM Utility", "HP pre-installed OEM bloatware", "Utilitário OEM pré-instalado da HP", "OEM & Third-Party", BloatSafety::Safe),
    kb!("DellInc.", "Dell OEM Utility", "Dell pre-installed OEM bloatware", "Utilitário OEM pré-instalado da Dell", "OEM & Third-Party", BloatSafety::Safe),
    kb!("E046963F.", "Lenovo OEM Utility", "Lenovo pre-installed OEM bloatware", "Utilitário OEM pré-instalado da Lenovo", "OEM & Third-Party", BloatSafety::Safe),
    kb!("B9ECED6F.", "ASUS OEM Utility", "ASUS pre-installed OEM bloatware", "Utilitário OEM pré-instalado da ASUS", "OEM & Third-Party", BloatSafety::Safe),
    kb!("AcerIncorporated.", "Acer OEM Utility", "Acer pre-installed OEM bloatware", "Utilitário OEM pré-instalado da Acer", "OEM & Third-Party", BloatSafety::Safe),
    kb!("9E2F88E3.Twitter", "X / Twitter (Preinstall)", "Pre-installed social web stub", "Atalho pré-instalado X / Twitter", "OEM & Third-Party", BloatSafety::Safe),
    kb!("C27EB4BA.DropboxOEM", "Dropbox OEM Offer", "OEM promotional Dropbox stub", "Atalho promocional OEM Dropbox", "OEM & Third-Party", BloatSafety::Safe),
];

/// Strictly protected core Windows runtime & shell packages — NEVER removable.
const PROTECTED_PREFIXES: &[&str] = &[
    "Microsoft.WindowsStore",
    "Microsoft.DesktopAppInstaller",
    "Microsoft.StorePurchaseApp",
    "Microsoft.VCLibs",
    "Microsoft.UI.Xaml",
    "Microsoft.NET.",
    "Microsoft.WindowsAppRuntime",
    "Microsoft.Services.Store",
    "Microsoft.WindowsTerminal",
    "Microsoft.WindowsCalculator",
    "Microsoft.WindowsNotepad",
    "Microsoft.ScreenSketch",
    "Microsoft.Windows.Photos",
    "Microsoft.SecHealthUI",
    "Microsoft.HEIFImageExtension",
    "Microsoft.HEVCVideoExtension",
    "Microsoft.VP9VideoExtensions",
    "Microsoft.WebMediaExtensions",
    "Microsoft.WebpImageExtension",
    "Microsoft.RawImageExtension",
    "Microsoft.AV1VideoExtension",
    "Microsoft.DirectX",
    "Microsoft.Winget.",
    "Microsoft.AAD.",
    "Microsoft.AccountsControl",
    "Microsoft.AsyncTextService",
    "Microsoft.BioEnrollment",
    "Microsoft.CredDialogHost",
    "Microsoft.ECApp",
    "Microsoft.LockApp",
    "Microsoft.Win32WebViewHost",
    "Microsoft.Windows.CloudExperienceHost",
    "Microsoft.Windows.ContentDeliveryManager",
    "Microsoft.Windows.OOBENetwork",
    "Microsoft.Windows.ParentalControls",
    "Microsoft.Windows.PeopleExperienceHost",
    "Microsoft.Windows.PinningConfirmationDialog",
    "Microsoft.Windows.Search",
    "Microsoft.Windows.ShellExperienceHost",
    "Microsoft.Windows.StartMenuExperienceHost",
    "Microsoft.Windows.XGpuEjectDialog",
    "MicrosoftWindows.Client.CBS",
    "MicrosoftWindows.Client.Core",
    "MicrosoftWindows.UndockedDevKit",
    "Windows.",
    "NVIDIACorp.",
    "AdvancedMicroDevices",
    "AppUp.Intel",
    "RealtekSemiconductorCorp",
    "DolbyLaboratories",
];

#[derive(Deserialize)]
struct RawAppx {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "PackageFullName")]
    full_name: Option<String>,
    #[serde(rename = "Version")]
    version: Option<String>,
    #[serde(rename = "IsFramework")]
    is_framework: Option<bool>,
    #[serde(rename = "NonRemovable")]
    non_removable: Option<bool>,
}

/// Scans installed Windows UWP / Appx packages.
/// When `include_all_removable` is true, surfaces ALL non-framework, removable Store packages
/// in addition to curated known bloatware so users have full visibility and control.
pub async fn scan_bloatware(include_all_removable: bool) -> Result<Vec<BloatPackage>, String> {
    let ps = "Get-AppxPackage | Select-Object Name, PackageFullName, Version, IsFramework, NonRemovable | ConvertTo-Json -Compress";
    let output = hidden_tokio_cmd("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", ps])
        .output()
        .await
        .map_err(|e| format!("Failed to query Appx packages: {e}"))?;

    let raw_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if raw_str.is_empty() {
        return Ok(Vec::new());
    }

    let parsed: Vec<RawAppx> = if raw_str.starts_with('[') {
        serde_json::from_str(&raw_str).unwrap_or_default()
    } else {
        serde_json::from_str::<RawAppx>(&raw_str)
            .map(|x| vec![x])
            .unwrap_or_default()
    };

    let mut found = Vec::new();
    for item in parsed {
        if item.is_framework.unwrap_or(false) || item.non_removable.unwrap_or(false) {
            continue;
        }
        let name = item.name.unwrap_or_default();
        let full_name = item.full_name.unwrap_or_default();
        let version = item.version.unwrap_or_default();
        if name.is_empty() || full_name.is_empty() {
            continue;
        }
        if PROTECTED_PREFIXES
            .iter()
            .any(|p| name.eq_ignore_ascii_case(p) || name.starts_with(p))
        {
            continue;
        }

        if let Some(known) = KNOWN_BLOAT
            .iter()
            .find(|k| name.eq_ignore_ascii_case(k.prefix) || name.starts_with(k.prefix))
        {
            found.push(BloatPackage {
                package_name: name,
                package_full_name: full_name,
                display_name: known.display.to_string(),
                desc_en: known.desc_en.to_string(),
                desc_pt: known.desc_pt.to_string(),
                category: known.category,
                safety: known.safety,
                selected: known.safety == BloatSafety::Safe,
                version,
                is_known_bloat: true,
            });
        } else if include_all_removable {
            let nice = name
                .split('.')
                .last()
                .unwrap_or(&name)
                .to_string();
            found.push(BloatPackage {
                package_name: name.clone(),
                package_full_name: full_name,
                display_name: nice,
                desc_en: format!("Installed UWP Store package ({name})"),
                desc_pt: format!("Pacote UWP instalado ({name})"),
                category: "Other Installed UWP",
                safety: BloatSafety::Moderate,
                selected: false,
                version,
                is_known_bloat: false,
            });
        }
    }

    found.sort_by(|a, b| {
        b.is_known_bloat
            .cmp(&a.is_known_bloat)
            .then_with(|| a.display_name.cmp(&b.display_name))
    });
    Ok(found)
}

pub async fn remove_bloat_packages(packages: Vec<BloatPackage>) -> (usize, usize) {
    let mut removed = 0usize;
    let mut failed = 0usize;

    for pkg in packages {
        let safe_full = pkg.package_full_name.replace('\'', "''");
        let safe_name = pkg.package_name.replace('\'', "''");
        let script = format!(
            "Remove-AppxPackage -Package '{safe_full}' -ErrorAction Stop; \
             Get-AppxProvisionedPackage -Online -ErrorAction SilentlyContinue | \
             Where-Object {{ $_.DisplayName -eq '{safe_name}' }} | \
             Remove-AppxProvisionedPackage -Online -ErrorAction SilentlyContinue | Out-Null"
        );
        match hidden_tokio_cmd("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
            .output()
            .await
        {
            Ok(out) if out.status.success() => removed += 1,
            _ => failed += 1,
        }
    }

    (removed, failed)
}

/// Executes deep system-level debloating operations beyond simple UWP package removal.
pub async fn run_deep_debloat_action(action: DeepDebloatAction) -> Result<String, String> {
    match action {
        DeepDebloatAction::RemoveOneDrive => {
            let script = r#"
Stop-Process -Name OneDrive -Force -ErrorAction SilentlyContinue
$sys64 = "$env:SystemRoot\SysWOW64\OneDriveSetup.exe"
$sys32 = "$env:SystemRoot\System32\OneDriveSetup.exe"
if (Test-Path $sys64) { Start-Process $sys64 -ArgumentList "/uninstall" -Wait -WindowStyle Hidden }
elseif (Test-Path $sys32) { Start-Process $sys32 -ArgumentList "/uninstall" -Wait -WindowStyle Hidden }
New-Item -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\OneDrive" -Force | Out-Null
Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\OneDrive" -Name "DisableFileSyncNGSC" -Value 1 -Type DWord -Force
"#;
            let out = hidden_tokio_cmd("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
                .output()
                .await
                .map_err(|e| e.to_string())?;
            if out.status.success() {
                Ok("OneDrive uninstalled and file sync disabled.".into())
            } else {
                Err("OneDrive uninstall encountered an error (verify Administrator rights).".into())
            }
        }
        DeepDebloatAction::RemoveWidgetsPack => {
            let script = r#"
Get-AppxPackage *WebExperience* | Remove-AppxPackage -ErrorAction SilentlyContinue
New-Item -Path "HKLM:\SOFTWARE\Policies\Microsoft\Dsh" -Force | Out-Null
Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Dsh" -Name "AllowNewsAndInterests" -Value 0 -Type DWord -Force
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced" -Name "TaskbarDa" -Value 0 -Type DWord -Force
"#;
            let _ = hidden_tokio_cmd("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
                .output()
                .await
                .map_err(|e| e.to_string())?;
            Ok("Windows Widgets & Web Experience Pack disabled.".into())
        }
        DeepDebloatAction::DisableRecallAndCopilot => {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            if let Ok((k, _)) = hkcu.create_subkey(r"Software\Policies\Microsoft\Windows\WindowsCopilot") {
                let _ = k.set_value("TurnOffWindowsCopilot", &1u32);
            }
            if let Ok((k, _)) = hklm.create_subkey(r"SOFTWARE\Policies\Microsoft\Windows\WindowsCopilot") {
                let _ = k.set_value("TurnOffWindowsCopilot", &1u32);
            }
            if let Ok((k, _)) = hkcu.create_subkey(r"Software\Policies\Microsoft\Windows\WindowsAI") {
                let _ = k.set_value("DisableAIDataAnalysis", &1u32);
            }
            if let Ok((k, _)) = hklm.create_subkey(r"SOFTWARE\Policies\Microsoft\Windows\WindowsAI") {
                let _ = k.set_value("DisableAIDataAnalysis", &1u32);
            }
            if let Ok((k, _)) = hkcu.create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced") {
                let _ = k.set_value("ShowCopilotButton", &0u32);
            }
            Ok("Windows Recall & Copilot AI policies disabled.".into())
        }
        DeepDebloatAction::BlockAutoInstalledBloat => {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let cdm = r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager";
            if let Ok((k, _)) = hkcu.create_subkey(cdm) {
                for name in [
                    "SilentInstalledAppsEnabled",
                    "OemPreInstalledAppsEnabled",
                    "PreInstalledAppsEnabled",
                    "PreInstalledAppsEverEnabled",
                    "ContentDeliveryAllowed",
                    "FeatureManagementEnabled",
                    "SubscribedContent-338388Enabled",
                    "SubscribedContent-338389Enabled",
                    "SubscribedContent-353694Enabled",
                    "SubscribedContent-353696Enabled",
                ] {
                    let _ = k.set_value(name, &0u32);
                }
            }
            if let Ok((k, _)) = hklm.create_subkey(r"SOFTWARE\Policies\Microsoft\Windows\CloudContent") {
                let _ = k.set_value("DisableWindowsConsumerFeatures", &1u32);
                let _ = k.set_value("DisableSoftLanding", &1u32);
            }
            Ok("Blocked Windows from silently auto-installing sponsored bloatware.".into())
        }
        DeepDebloatAction::RemoveLegacyCapabilities => {
            let script = r#"
$caps = @("App.StepsRecorder*", "MathRecognizer*", "Print.Fax.Scan*", "App.Support.QuickAssist*", "Microsoft.Windows.WordPad*", "Microsoft.Windows.PowerShell.ISE*")
foreach ($pat in $caps) {
    Get-WindowsCapability -Online -Name $pat -ErrorAction SilentlyContinue | Where-Object { $_.State -eq 'Installed' } | Remove-WindowsCapability -Online -ErrorAction SilentlyContinue | Out-Null
}
"#;
            let _ = hidden_tokio_cmd("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
                .output()
                .await
                .map_err(|e| e.to_string())?;
            Ok("Removed installed legacy Windows optional capabilities.".into())
        }
        DeepDebloatAction::CleanTelemetryTasks => {
            let script = r#"
$tasks = @(
    "\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser",
    "\Microsoft\Windows\Application Experience\ProgramDataUpdater",
    "\Microsoft\Windows\Customer Experience Improvement Program\Consolidator",
    "\Microsoft\Windows\Customer Experience Improvement Program\UsbCeip",
    "\Microsoft\Windows\DiskDiagnostic\Microsoft-Windows-DiskDiagnosticDataCollector",
    "\Microsoft\Windows\Feedback\Siuf\DmClient",
    "\Microsoft\Windows\Feedback\Siuf\DmClientOnScenarioDownload"
)
foreach ($t in $tasks) { schtasks /Change /TN $t /Disable 2>$null | Out-Null }
"#;
            let _ = hidden_tokio_cmd("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
                .output()
                .await
                .map_err(|e| e.to_string())?;
            Ok("Disabled background Windows telemetry & CEIP scheduled tasks.".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debloater_has_rich_rules_and_protects_critical_system_packages() {
        assert!(KNOWN_BLOAT.len() >= 65);
        assert_eq!(DEEP_DEBLOAT_TOOLS.len(), 6);
        for critical in [
            "Microsoft.WindowsStore",
            "Microsoft.DesktopAppInstaller",
            "Microsoft.SecHealthUI",
            "Microsoft.WindowsTerminal",
        ] {
            assert!(
                PROTECTED_PREFIXES
                    .iter()
                    .any(|p| critical.eq_ignore_ascii_case(p) || critical.starts_with(p)),
                "Critical package {critical} must be protected"
            );
        }
    }
}
