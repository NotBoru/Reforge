use crate::core::winapi::hidden_tokio_cmd;
use serde::Deserialize;

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
}

struct KnownBloat {
    prefix: &'static str,
    display: &'static str,
    desc_en: &'static str,
    desc_pt: &'static str,
    category: &'static str,
    safety: BloatSafety,
}

const KNOWN_BLOAT: &[KnownBloat] = &[
    KnownBloat {
        prefix: "Clipchamp.Clipchamp",
        display: "Microsoft Clipchamp",
        desc_en: "Pre-installed cloud video editor",
        desc_pt: "Editor de vídeo pré-instalado na nuvem",
        category: "Microsoft",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.BingNews",
        display: "Microsoft News",
        desc_en: "Bing News feed widget & app",
        desc_pt: "Aplicativo e feed de notícias Bing",
        category: "Bing & Feed",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.BingWeather",
        display: "MSN Weather",
        desc_en: "Bing Weather pre-installed app",
        desc_pt: "Aplicativo pré-instalado de clima",
        category: "Bing & Feed",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.BingSearch",
        display: "Microsoft Bing Search",
        desc_en: "Web search integration package",
        desc_pt: "Pacote de integração de busca Bing",
        category: "Bing & Feed",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.MicrosoftSolitaireCollection",
        display: "Microsoft Solitaire Collection",
        desc_en: "Ad-supported bundled card games",
        desc_pt: "Coleção de jogos Paciência com anúncios",
        category: "Games",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.549981C3F5F10",
        display: "Cortana",
        desc_en: "Legacy deprecated voice assistant",
        desc_pt: "Assistente de voz descontinuada Cortana",
        category: "Microsoft",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.Copilot",
        display: "Microsoft Copilot",
        desc_en: "Bundled Windows Copilot web wrapper",
        desc_pt: "Aplicativo pré-instalado do Copilot",
        category: "AI & Web",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.WindowsFeedbackHub",
        display: "Feedback Hub",
        desc_en: "Windows telemetry & diagnostic feedback app",
        desc_pt: "App de envio de feedback e diagnóstico",
        category: "Microsoft",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.GetHelp",
        display: "Get Help",
        desc_en: "Microsoft support chat wrapper",
        desc_pt: "Atalho de suporte da Microsoft",
        category: "Microsoft",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.Getstarted",
        display: "Microsoft Tips",
        desc_en: "Windows onboarding tips notifications",
        desc_pt: "Dicas e notificações iniciais do Windows",
        category: "Microsoft",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.MicrosoftOfficeHub",
        display: "Microsoft 365 (Office Hub)",
        desc_en: "Promotional Microsoft 365 launcher stub",
        desc_pt: "Atalho promocional do Microsoft 365",
        category: "Microsoft",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.People",
        display: "Microsoft People",
        desc_en: "Unused contacts integration app",
        desc_pt: "App de contatos raramente utilizado",
        category: "Microsoft",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.SkypeApp",
        display: "Skype",
        desc_en: "Pre-installed Skype UWP client",
        desc_pt: "Cliente UWP pré-instalado do Skype",
        category: "Communication",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "MSTeams",
        display: "Microsoft Teams (Personal)",
        desc_en: "Bundled Windows 11 Teams consumer app",
        desc_pt: "Microsoft Teams pré-instalado no Windows 11",
        category: "Communication",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "MicrosoftTeams",
        display: "Microsoft Teams Chat",
        desc_en: "Legacy Teams Chat integration",
        desc_pt: "Integração legada de Chat do Teams",
        category: "Communication",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.Todos",
        display: "Microsoft To Do",
        desc_en: "Bundled task list client",
        desc_pt: "Lista de tarefas da Microsoft",
        category: "Microsoft",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.PowerAutomateDesktop",
        display: "Power Automate Desktop",
        desc_en: "Enterprise workflow automation stub",
        desc_pt: "Automação corporativa pré-instalada",
        category: "Microsoft",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.ZuneVideo",
        display: "Movies & TV",
        desc_en: "Legacy Microsoft video store player",
        desc_pt: "Reprodutor Filmes e TV da Microsoft",
        category: "Media",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.ZuneMusic",
        display: "Media Player (Groove)",
        desc_en: "Built-in Windows music player",
        desc_pt: "Reprodutor de música nativo do Windows",
        category: "Media",
        safety: BloatSafety::Moderate,
    },
    KnownBloat {
        prefix: "Microsoft.YourPhone",
        display: "Phone Link",
        desc_en: "Android/iOS phone syncing background app",
        desc_pt: "Sincronização em segundo plano com celular",
        category: "Microsoft",
        safety: BloatSafety::Moderate,
    },
    KnownBloat {
        prefix: "Microsoft.OutlookForWindows",
        display: "Outlook (New)",
        desc_en: "WebView-based replacement Mail client",
        desc_pt: "Novo cliente web do Outlook pré-instalado",
        category: "Communication",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.WindowsCommunicationsApps",
        display: "Mail & Calendar",
        desc_en: "Legacy Windows Mail & Calendar suite",
        desc_pt: "Email e Calendário legados do Windows",
        category: "Communication",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.MicrosoftStickyNotes",
        display: "Sticky Notes",
        desc_en: "Desktop sticky notes UWP app",
        desc_pt: "Notas autoadesivas do Windows",
        category: "Microsoft",
        safety: BloatSafety::Moderate,
    },
    KnownBloat {
        prefix: "Microsoft.WindowsMaps",
        display: "Windows Maps",
        desc_en: "Pre-installed offline maps app",
        desc_pt: "Aplicativo de mapas da Microsoft",
        category: "Microsoft",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.WindowsSoundRecorder",
        display: "Sound Recorder",
        desc_en: "Basic Windows voice recorder",
        desc_pt: "Gravador de voz básico do Windows",
        category: "Media",
        safety: BloatSafety::Moderate,
    },
    KnownBloat {
        prefix: "MicrosoftCorporationII.QuickAssist",
        display: "Quick Assist",
        desc_en: "Remote assistance support tool",
        desc_pt: "Assistência remota da Microsoft",
        category: "Microsoft",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "MicrosoftCorporationII.MicrosoftFamily",
        display: "Microsoft Family Safety",
        desc_en: "Parental controls stub app",
        desc_pt: "App de controle parental Microsoft",
        category: "Microsoft",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.Windows.DevHome",
        display: "Dev Home",
        desc_en: "Deprecated developer dashboard app",
        desc_pt: "Dashboard Dev Home descontinuado",
        category: "Microsoft",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.XboxGamingOverlay",
        display: "Xbox Game Bar",
        desc_en: "In-game Xbox overlay (remove if you don't use Game Bar)",
        desc_pt: "Overlay do Xbox em jogos (remova se não usar Game Bar)",
        category: "Xbox",
        safety: BloatSafety::Moderate,
    },
    KnownBloat {
        prefix: "Microsoft.Xbox.TCUI",
        display: "Xbox Live In-Game",
        desc_en: "Xbox Live UI integration component",
        desc_pt: "Integração de interface Xbox Live",
        category: "Xbox",
        safety: BloatSafety::Moderate,
    },
    KnownBloat {
        prefix: "Microsoft.XboxSpeechToTextOverlay",
        display: "Xbox Speech To Text",
        desc_en: "Xbox accessibility speech overlay",
        desc_pt: "Overlay de voz para texto do Xbox",
        category: "Xbox",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Microsoft.GamingApp",
        display: "Xbox App",
        desc_en: "Xbox Game Pass PC storefront",
        desc_pt: "Aplicativo Xbox / Game Pass",
        category: "Xbox",
        safety: BloatSafety::Moderate,
    },
    KnownBloat {
        prefix: "Microsoft.MixedReality.Portal",
        display: "Mixed Reality Portal",
        desc_en: "Deprecated Windows Mixed Reality VR portal",
        desc_pt: "Portal VR descontinuado do Windows",
        category: "Microsoft",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "SpotifyAB.SpotifyMusic",
        display: "Spotify (Store Preinstall)",
        desc_en: "OEM / Microsoft Store Spotify stub",
        desc_pt: "Versão da loja pré-instalada do Spotify",
        category: "Third-Party",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Disney.37853FC22B2CE",
        display: "Disney+ (Preinstall)",
        desc_en: "Pre-installed streaming stub",
        desc_pt: "Atalho pré-instalado do Disney+",
        category: "Third-Party",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "BytedancePte.Ltd.TikTok",
        display: "TikTok (Preinstall)",
        desc_en: "Pre-installed social media stub",
        desc_pt: "Atalho pré-instalado do TikTok",
        category: "Third-Party",
        safety: BloatSafety::Safe,
    },
    KnownBloat {
        prefix: "Facebook.InstagramBeta",
        display: "Instagram (Preinstall)",
        desc_en: "Pre-installed social media stub",
        desc_pt: "Atalho pré-instalado do Instagram",
        category: "Third-Party",
        safety: BloatSafety::Safe,
    },
];

/// Hardcoded system protection allowlist — NEVER surfaced for removal.
const PROTECTED_PREFIXES: &[&str] = &[
    "Microsoft.WindowsStore",
    "Microsoft.DesktopAppInstaller",
    "Microsoft.StorePurchaseApp",
    "Microsoft.VCLibs",
    "Microsoft.UI.Xaml",
    "Microsoft.NET.",
    "Microsoft.WindowsTerminal",
    "Microsoft.WindowsCalculator",
    "Microsoft.WindowsNotepad",
    "Microsoft.ScreenSketch",
    "Microsoft.Windows.Photos",
    "Microsoft.SecHealthUI",
    "Microsoft.HEIFImageExtension",
    "Microsoft.VP9VideoExtensions",
    "Microsoft.WebMediaExtensions",
    "Microsoft.WebpImageExtension",
];

#[derive(Deserialize)]
struct RawAppx {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "PackageFullName")]
    full_name: Option<String>,
    #[serde(rename = "Version")]
    version: Option<String>,
}

/// Scans installed Windows UWP / Appx packages in a single fast PowerShell JSON batch
/// and matches against the curated safe bloatware catalog.
pub async fn scan_bloatware() -> Result<Vec<BloatPackage>, String> {
    let ps = "Get-AppxPackage | Select-Object Name, PackageFullName, Version | ConvertTo-Json -Compress";
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
            });
        }
    }

    found.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    Ok(found)
}

/// Removes selected bloatware packages (both current user and provisioned image when elevated).
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
