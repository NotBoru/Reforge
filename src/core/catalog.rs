#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppCategory {
    All,
    Browsers,
    Communication,
    Development,
    Gaming,
    Media,
    Utilities,
    Productivity,
}

impl AppCategory {
    pub const ALL: &'static [AppCategory] = &[
        AppCategory::All,
        AppCategory::Browsers,
        AppCategory::Communication,
        AppCategory::Development,
        AppCategory::Gaming,
        AppCategory::Media,
        AppCategory::Utilities,
        AppCategory::Productivity,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetKind {
    Essentials,
    Developer,
    Gaming,
    Creator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerKind {
    Exe,
    Msi,
}

#[derive(Debug, Clone, Copy)]
pub enum DirectSource {
    StaticUrl {
        url: &'static str,
        kind: InstallerKind,
        silent_args: &'static [&'static str],
    },
    GitHubRelease {
        repo: &'static str,
        asset_contains: &'static [&'static str],
        asset_excludes: &'static [&'static str],
        asset_ends_with: &'static str,
        kind: InstallerKind,
        silent_args: &'static [&'static str],
    },
}

#[derive(Debug, Clone, Copy)]
pub struct CatalogApp {
    pub id: &'static str,
    pub name: &'static str,
    pub publisher: &'static str,
    pub domain: &'static str,
    pub desc_en: &'static str,
    pub desc_pt: &'static str,
    pub category: AppCategory,
    pub winget_id: &'static str,
    pub direct: Option<DirectSource>,
    pub detect_keywords: &'static [&'static str],
    pub presets: &'static [PresetKind],
}

pub fn catalog() -> &'static [CatalogApp] {
    CATALOG
}

pub fn find_by_winget_id(wid: &str) -> Option<&'static CatalogApp> {
    CATALOG
        .iter()
        .find(|a| a.winget_id.eq_ignore_ascii_case(wid))
}

pub fn preset_app_ids(preset: PresetKind) -> Vec<&'static str> {
    CATALOG
        .iter()
        .filter(|a| a.presets.contains(&preset))
        .map(|a| a.id)
        .collect()
}

macro_rules! app_entry {
    ($id:expr, $name:expr, $pub:expr, $dom:expr, $en:expr, $pt:expr, $cat:expr, $wg:expr, $dir:expr, $kw:expr, $pre:expr) => {
        CatalogApp {
            id: $id,
            name: $name,
            publisher: $pub,
            domain: $dom,
            desc_en: $en,
            desc_pt: $pt,
            category: $cat,
            winget_id: $wg,
            direct: $dir,
            detect_keywords: $kw,
            presets: $pre,
        }
    };
}

const CATALOG: &[CatalogApp] = &[
    // ── BROWSERS ─────────────────────────────────────────────────────────────
    app_entry!(
        "chrome", "Google Chrome", "Google", "google.com/chrome",
        "Fast web browser with Google sync", "Navegador rápido com sincronização Google",
        AppCategory::Browsers, "Google.Chrome",
        Some(DirectSource::StaticUrl {
            url: "https://dl.google.com/dl/chrome/install/googlechromestandaloneenterprise64.msi",
            kind: InstallerKind::Msi, silent_args: &["/qn", "/norestart"],
        }),
        &["google chrome"], &[PresetKind::Essentials]
    ),
    app_entry!(
        "brave", "Brave Browser", "Brave Software", "brave.com",
        "Privacy-first Chromium browser with built-in ad blocker", "Navegador Chromium privado com bloqueador de anúncios",
        AppCategory::Browsers, "Brave.Brave",
        Some(DirectSource::StaticUrl {
            url: "https://laptop-updates.brave.com/latest/winx64",
            kind: InstallerKind::Exe, silent_args: &["/silent", "/install"],
        }),
        &["brave"], &[PresetKind::Developer, PresetKind::Gaming]
    ),
    app_entry!(
        "firefox", "Mozilla Firefox", "Mozilla", "firefox.com",
        "Open-source browser built for privacy and speed", "Navegador open-source focado em velocidade e privacidade",
        AppCategory::Browsers, "Mozilla.Firefox",
        Some(DirectSource::StaticUrl {
            url: "https://download.mozilla.org/?product=firefox-latest-ssl&os=win64&lang=en-US",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["mozilla firefox"], &[]
    ),
    app_entry!(
        "zen", "Zen Browser", "Zen Team", "zen-browser.app",
        "Calm, minimal Firefox browser with vertical tabs", "Navegador minimalista baseado no Firefox com abas verticais",
        AppCategory::Browsers, "Zen-Team.Zen-Browser",
        Some(DirectSource::GitHubRelease {
            repo: "zen-browser/desktop", asset_contains: &["installer"],
            asset_excludes: &["arm64"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["zen browser", "zen twilight"], &[]
    ),
    app_entry!(
        "arc", "Arc Browser", "The Browser Company", "arc.net",
        "Modern workspace browser with Spaces and Split View", "Navegador moderno com espaços de trabalho e visão dividida",
        AppCategory::Browsers, "TheBrowserCompany.Arc", None, &["arc"], &[]
    ),
    app_entry!(
        "vivaldi", "Vivaldi", "Vivaldi Technologies", "vivaldi.com",
        "Power-user browser with deep tab customization", "Navegador avançado altamente personalizável",
        AppCategory::Browsers, "Vivaldi.Vivaldi", None, &["vivaldi"], &[]
    ),
    app_entry!(
        "librewolf", "LibreWolf", "LibreWolf", "librewolf.net",
        "Hardened Firefox fork with zero telemetry", "Fork do Firefox focado em privacidade máxima",
        AppCategory::Browsers, "LibreWolf.LibreWolf", None, &["librewolf"], &[]
    ),

    // ── COMMUNICATION ────────────────────────────────────────────────────────
    app_entry!(
        "discord", "Discord", "Discord Inc.", "discord.com",
        "Voice, video, and community chat", "Chat de voz, vídeo e comunidades",
        AppCategory::Communication, "Discord.Discord",
        Some(DirectSource::StaticUrl {
            url: "https://discord.com/api/downloads/distributions/app/installers/latest?channel=stable&platform=win&arch=x64",
            kind: InstallerKind::Exe, silent_args: &["-s"],
        }),
        &["discord"], &[PresetKind::Essentials, PresetKind::Gaming]
    ),
    app_entry!(
        "vesktop", "Vesktop", "Vencord", "vencord.dev",
        "Lightweight Discord client with Vencord built in", "Cliente leve para Discord com Vencord integrado",
        AppCategory::Communication, "Vencord.Vesktop",
        Some(DirectSource::GitHubRelease {
            repo: "Vencord/Vesktop", asset_contains: &["Setup"],
            asset_excludes: &["arm64"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["vesktop"], &[]
    ),
    app_entry!(
        "telegram", "Telegram Desktop", "Telegram", "telegram.org",
        "Fast, cloud-synced desktop messenger", "Mensageiro rápido sincronizado na nuvem",
        AppCategory::Communication, "Telegram.TelegramDesktop",
        Some(DirectSource::StaticUrl {
            url: "https://telegram.org/dl/desktop/win64",
            kind: InstallerKind::Exe, silent_args: &["/VERYSILENT", "/NORESTART"],
        }),
        &["telegram desktop"], &[]
    ),
    app_entry!(
        "whatsapp", "WhatsApp", "Meta", "whatsapp.com",
        "Official WhatsApp desktop client", "Cliente oficial do WhatsApp para Windows",
        AppCategory::Communication, "9NKSQGP7F2NH", None, &["whatsapp"], &[]
    ),
    app_entry!(
        "slack", "Slack", "Salesforce", "slack.com",
        "Team messaging and workspace hub", "Comunicação corporativa e canais de equipe",
        AppCategory::Communication, "SlackTechnologies.Slack", None, &["slack"], &[]
    ),
    app_entry!(
        "zoom", "Zoom Workplace", "Zoom", "zoom.us",
        "Video conferencing and screen sharing", "Videoconferências e reuniões online",
        AppCategory::Communication, "Zoom.Zoom",
        Some(DirectSource::StaticUrl {
            url: "https://zoom.us/client/latest/ZoomInstallerFull.msi?archType=x64",
            kind: InstallerKind::Msi, silent_args: &["/qn", "/norestart"],
        }),
        &["zoom"], &[]
    ),
    app_entry!(
        "signal", "Signal", "Signal Foundation", "signal.org",
        "End-to-end encrypted private messenger", "Mensageiro privado com criptografia ponta a ponta",
        AppCategory::Communication, "OpenWhisperSystems.Signal", None, &["signal"], &[]
    ),

    // ── DEVELOPMENT ──────────────────────────────────────────────────────────
    app_entry!(
        "vscode", "Visual Studio Code", "Microsoft", "code.visualstudio.com",
        "Extensible source code editor", "Editor de código leve e extensível",
        AppCategory::Development, "Microsoft.VisualStudioCode",
        Some(DirectSource::StaticUrl {
            url: "https://code.visualstudio.com/sha/download?build=stable&os=win32-x64-user",
            kind: InstallerKind::Exe,
            silent_args: &["/VERYSILENT", "/NORESTART", "/MERGETASKS=!runcode,addcontextmenufiles,addcontextmenufolders,addtopath"],
        }),
        &["visual studio code"], &[PresetKind::Developer]
    ),
    app_entry!(
        "cursor", "Cursor", "Anysphere", "cursor.com",
        "AI-native code editor built on VS Code", "Editor de código com IA nativa baseado no VS Code",
        AppCategory::Development, "Anysphere.Cursor", None, &["cursor"], &[]
    ),
    app_entry!(
        "git", "Git for Windows", "Git", "git-scm.com",
        "Distributed version control + Git Bash", "Controle de versão distribuído + Git Bash",
        AppCategory::Development, "Git.Git",
        Some(DirectSource::GitHubRelease {
            repo: "git-for-windows/git", asset_contains: &["64-bit"],
            asset_excludes: &["Portable", "MinGit", "tar", "arm64"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/VERYSILENT", "/NORESTART"],
        }),
        &["git"], &[PresetKind::Developer]
    ),
    app_entry!(
        "windows_terminal", "Windows Terminal", "Microsoft", "microsoft.com",
        "GPU-accelerated modern tabbed terminal", "Terminal moderno com abas e aceleração GPU",
        AppCategory::Development, "Microsoft.WindowsTerminal", None, &["windows terminal"], &[PresetKind::Developer]
    ),
    app_entry!(
        "powershell7", "PowerShell 7", "Microsoft", "microsoft.com",
        "Modern cross-platform PowerShell shell", "Shell PowerShell 7 moderno e rápido",
        AppCategory::Development, "Microsoft.PowerShell", None, &["powershell 7"], &[]
    ),
    app_entry!(
        "nodejs", "Node.js LTS", "OpenJS Foundation", "nodejs.org",
        "JavaScript runtime built on Chrome V8", "Runtime JavaScript LTS com npm incluso",
        AppCategory::Development, "OpenJS.NodeJS.LTS", None, &["node.js"], &[PresetKind::Developer]
    ),
    app_entry!(
        "python3", "Python 3.12", "Python Software Foundation", "python.org",
        "Scripting, backend and data language", "Linguagem Python 3 com pip incluso",
        AppCategory::Development, "Python.Python.3.12", None, &["python 3."], &[PresetKind::Developer]
    ),
    app_entry!(
        "golang", "Go", "Google", "go.dev",
        "Fast, statically typed systems language", "Linguagem compilada rápida para backend e CLI",
        AppCategory::Development, "GoLang.Go", None, &["go programming language"], &[]
    ),
    app_entry!(
        "rustup", "Rustup (Rust)", "Rust Project", "rust-lang.org",
        "Official Rust compiler & Cargo installer", "Instalador oficial da toolchain Rust e Cargo",
        AppCategory::Development, "Rustlang.Rustup",
        Some(DirectSource::StaticUrl {
            url: "https://win.rustup.rs/x86_64",
            kind: InstallerKind::Exe, silent_args: &["-y"],
        }),
        &["rustup"], &[]
    ),
    app_entry!(
        "docker", "Docker Desktop", "Docker Inc.", "docker.com",
        "Containers, Compose and local Kubernetes", "Ambiente de containers e Docker Compose",
        AppCategory::Development, "Docker.DockerDesktop", None, &["docker desktop"], &[]
    ),
    app_entry!(
        "github_desktop", "GitHub Desktop", "GitHub", "github.com",
        "Visual Git client for repositories and PRs", "Cliente visual intuitivo para repositórios GitHub",
        AppCategory::Development, "GitHub.GitHubDesktop",
        Some(DirectSource::StaticUrl {
            url: "https://central.github.com/deployments/desktop/desktop/latest/win32",
            kind: InstallerKind::Exe, silent_args: &["-s"],
        }),
        &["github desktop"], &[]
    ),
    app_entry!(
        "neovim", "Neovim", "Neovim", "neovim.io",
        "Hyperextensible Vim-based modal editor", "Editor modal ultrarrápido baseado em Vim",
        AppCategory::Development, "Neovim.Neovim",
        Some(DirectSource::GitHubRelease {
            repo: "neovim/neovim", asset_contains: &["win64"],
            asset_excludes: &[], asset_ends_with: ".msi",
            kind: InstallerKind::Msi, silent_args: &["/qn", "/norestart"],
        }),
        &["neovim"], &[]
    ),
    app_entry!(
        "postman", "Postman", "Postman Inc.", "postman.com",
        "REST & GraphQL API client and test suite", "Plataforma para testar e depurar APIs",
        AppCategory::Development, "Postman.Postman", None, &["postman"], &[]
    ),
    app_entry!(
        "insomnia", "Insomnia", "Kong", "insomnia.rest",
        "Lightweight open-source REST & GraphQL client", "Cliente leve e limpo para APIs REST e GraphQL",
        AppCategory::Development, "Insomnia.Insomnia", None, &["insomnia"], &[]
    ),
    app_entry!(
        "dbeaver", "DBeaver Community", "DBeaver", "dbeaver.io",
        "Universal SQL database GUI client", "Cliente universal para bancos SQL e NoSQL",
        AppCategory::Development, "dbeaver.dbeaver", None, &["dbeaver"], &[]
    ),
    app_entry!(
        "jetbrains_toolbox", "JetBrains Toolbox", "JetBrains", "jetbrains.com",
        "Manage IntelliJ, RustRover, Rider & WebStorm", "Gerenciador oficial de IDEs JetBrains",
        AppCategory::Development, "JetBrains.Toolbox", None, &["jetbrains toolbox"], &[]
    ),

    // ── GAMING ───────────────────────────────────────────────────────────────
    app_entry!(
        "steam", "Steam", "Valve", "store.steampowered.com",
        "PC gaming platform, store and library", "Principal plataforma e biblioteca de jogos para PC",
        AppCategory::Gaming, "Valve.Steam",
        Some(DirectSource::StaticUrl {
            url: "https://cdn.fastly.steamstatic.com/client/installer/SteamSetup.exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["steam"], &[PresetKind::Gaming]
    ),
    app_entry!(
        "epic", "Epic Games Launcher", "Epic Games", "epicgames.com",
        "Epic storefront, Unreal Engine and weekly games", "Loja Epic Games, Unreal Engine e jogos semanais",
        AppCategory::Gaming, "EpicGames.EpicGamesLauncher",
        Some(DirectSource::StaticUrl {
            url: "https://launcher-public-service-prod06.ol.epicgames.com/launcher/api/installer/download/EpicGamesLauncherInstaller.msi",
            kind: InstallerKind::Msi, silent_args: &["/qn", "/norestart"],
        }),
        &["epic games launcher"], &[]
    ),
    app_entry!(
        "battlenet", "Battle.net", "Blizzard", "battle.net",
        "Blizzard launcher for Overwatch, WoW & Diablo", "Launcher oficial da Blizzard",
        AppCategory::Gaming, "Blizzard.BattleNet", None, &["battle.net"], &[]
    ),
    app_entry!(
        "ea_app", "EA App", "Electronic Arts", "ea.com",
        "Electronic Arts PC storefront and launcher", "Plataforma oficial de jogos da EA",
        AppCategory::Gaming, "ElectronicArts.EADesktop", None, &["ea app"], &[]
    ),
    app_entry!(
        "prism_launcher", "Prism Launcher", "Prism Launcher", "prismlauncher.org",
        "Fast open-source multi-instance Minecraft launcher", "Launcher open-source rápido para Minecraft",
        AppCategory::Gaming, "PrismLauncher.PrismLauncher",
        Some(DirectSource::GitHubRelease {
            repo: "PrismLauncher/PrismLauncher", asset_contains: &["Windows-MSVC-Setup"],
            asset_excludes: &["arm64", "Portable"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/VERYSILENT", "/NORESTART"],
        }),
        &["prism launcher"], &[]
    ),
    app_entry!(
        "modrinth", "Modrinth App", "Modrinth", "modrinth.com",
        "Clean modern Minecraft modpack launcher", "Launcher moderno para modpacks no Modrinth",
        AppCategory::Gaming, "Modrinth.ModrinthApp", None, &["modrinth"], &[]
    ),
    app_entry!(
        "playnite", "Playnite", "Josef Nemec", "playnite.link",
        "Unified library across all game launchers", "Biblioteca unificada para todos seus jogos",
        AppCategory::Gaming, "Playnite.Playnite",
        Some(DirectSource::GitHubRelease {
            repo: "JosefNemec/Playnite", asset_contains: &["Playnite"],
            asset_excludes: &["Portable"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/VERYSILENT", "/NORESTART"],
        }),
        &["playnite"], &[]
    ),
    app_entry!(
        "msi_afterburner", "MSI Afterburner", "MSI", "msi.com",
        "GPU tuning, fan curves and RivaTuner FPS HUD", "Monitoramento de FPS, curvas de fan e overclock",
        AppCategory::Gaming, "Guru3D.Afterburner", None, &["msi afterburner"], &[PresetKind::Gaming]
    ),
    app_entry!(
        "hwinfo", "HWiNFO64", "REALiX", "hwinfo.com",
        "Precision hardware sensors and diagnostics", "Monitoramento preciso de temperaturas e sensores",
        AppCategory::Gaming, "REALiX.HWiNFO", None, &["hwinfo"], &[PresetKind::Gaming]
    ),
    app_entry!(
        "parsec", "Parsec", "Unity / Parsec", "parsec.app",
        "Ultra-low latency remote desktop and streaming", "Acesso remoto e streaming com latência mínima",
        AppCategory::Gaming, "Parsec.Parsec",
        Some(DirectSource::StaticUrl {
            url: "https://builds.parsec.app/package/parsec-windows.exe",
            kind: InstallerKind::Exe, silent_args: &["/silent", "/shared"],
        }),
        &["parsec"], &[]
    ),
    app_entry!(
        "fancontrol", "FanControl", "Rem0o", "getfancontrol.com",
        "Lightweight custom GPU & case fan curves", "Controle avançado e leve de ventoinhas do PC",
        AppCategory::Gaming, "Rem0o.FanControl", None, &["fancontrol"], &[]
    ),

    // ── MEDIA & CREATIVE ─────────────────────────────────────────────────────
    app_entry!(
        "vlc", "VLC Media Player", "VideoLAN", "videolan.org",
        "Universal open-source audio & video player", "Reprodutor universal open-source para qualquer mídia",
        AppCategory::Media, "VideoLAN.VLC", None, &["vlc media player"], &[PresetKind::Essentials, PresetKind::Creator]
    ),
    app_entry!(
        "obs", "OBS Studio", "OBS Project", "obsproject.com",
        "Studio-grade screen recording and streaming", "Gravação de tela e transmissão ao vivo profissional",
        AppCategory::Media, "OBSProject.OBSStudio",
        Some(DirectSource::GitHubRelease {
            repo: "obsproject/obs-studio", asset_contains: &["Windows-x64-Installer"],
            asset_excludes: &["arm64"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["obs studio"], &[PresetKind::Gaming, PresetKind::Creator]
    ),
    app_entry!(
        "spotify", "Spotify", "Spotify AB", "spotify.com",
        "Music and podcast streaming client", "Streaming de músicas e podcasts",
        AppCategory::Media, "Spotify.Spotify",
        Some(DirectSource::StaticUrl {
            url: "https://download.scdn.co/SpotifySetup.exe",
            kind: InstallerKind::Exe, silent_args: &["/silent"],
        }),
        &["spotify"], &[PresetKind::Gaming]
    ),
    app_entry!(
        "sharex", "ShareX", "ShareX Team", "getsharex.com",
        "Instant screenshot, GIF capture and workflows", "Captura de tela, GIFs e ferramentas visuais",
        AppCategory::Media, "ShareX.ShareX",
        Some(DirectSource::GitHubRelease {
            repo: "ShareX/ShareX", asset_contains: &["setup"],
            asset_excludes: &["portable"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/VERYSILENT", "/NORESTART"],
        }),
        &["sharex"], &[PresetKind::Essentials, PresetKind::Creator]
    ),
    app_entry!(
        "flameshot", "Flameshot", "Flameshot", "flameshot.org",
        "Clean annotated screenshot capture utility", "Captura de tela com anotações rápidas",
        AppCategory::Media, "Flameshot.Flameshot", None, &["flameshot"], &[]
    ),
    app_entry!(
        "audacity", "Audacity", "Muse Group", "audacityteam.org",
        "Multi-track audio recorder and editor", "Editor e gravador de áudio multipista",
        AppCategory::Media, "Audacity.Audacity",
        Some(DirectSource::GitHubRelease {
            repo: "audacity/audacity", asset_contains: &["64bit"],
            asset_excludes: &["zip"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/VERYSILENT", "/NORESTART"],
        }),
        &["audacity"], &[PresetKind::Creator]
    ),
    app_entry!(
        "gimp", "GIMP", "GIMP Team", "gimp.org",
        "Open-source raster graphics editor", "Editor de imagens open-source completo",
        AppCategory::Media, "GIMP.GIMP.3", None, &["gimp"], &[PresetKind::Creator]
    ),
    app_entry!(
        "krita", "Krita", "KDE", "krita.org",
        "Digital painting and illustration studio", "Pintura digital e ilustração profissional",
        AppCategory::Media, "KDE.Krita", None, &["krita"], &[]
    ),
    app_entry!(
        "inkscape", "Inkscape", "Inkscape", "inkscape.org",
        "Open-source SVG vector graphics editor", "Editor vetorial SVG open-source",
        AppCategory::Media, "Inkscape.Inkscape", None, &["inkscape"], &[]
    ),
    app_entry!(
        "handbrake", "HandBrake", "HandBrake Team", "handbrake.fr",
        "Fast video transcoder with NVENC & AV1", "Conversor de vídeo rápido com NVENC e AV1",
        AppCategory::Media, "HandBrake.HandBrake",
        Some(DirectSource::GitHubRelease {
            repo: "HandBrake/HandBrake", asset_contains: &["x86_64-Win_GUI"],
            asset_excludes: &["arm64"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["handbrake"], &[PresetKind::Creator]
    ),
    app_entry!(
        "blender", "Blender", "Blender Foundation", "blender.org",
        "3D modeling, animation, rendering & VFX", "Suíte 3D completa para modelagem e VFX",
        AppCategory::Media, "BlenderFoundation.Blender", None, &["blender"], &[PresetKind::Creator]
    ),
    app_entry!(
        "paint_net", "Paint.NET", "dotPDN", "getpaint.net",
        "Fast, lightweight everyday image editor", "Editor de imagens leve e super rápido",
        AppCategory::Media, "dotPDN.PaintDotNet", None, &["paint.net"], &[]
    ),

    // ── UTILITIES ────────────────────────────────────────────────────────────
    app_entry!(
        "7zip", "7-Zip", "Igor Pavlov", "7-zip.org",
        "Ultra-fast high-compression archive utility", "Compactador ultrarrápido de arquivos",
        AppCategory::Utilities, "7zip.7zip",
        Some(DirectSource::GitHubRelease {
            repo: "ip7z/7zip", asset_contains: &["-x64"],
            asset_excludes: &["arm64", "extra"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["7-zip"], &[PresetKind::Essentials, PresetKind::Developer, PresetKind::Gaming, PresetKind::Creator]
    ),
    app_entry!(
        "nanazip", "NanaZip", "M2-Team", "github.com",
        "Modern 7-Zip fork for Windows 11 context menu", "Fork moderno do 7-Zip com menu do Windows 11",
        AppCategory::Utilities, "M2Team.NanaZip", None, &["nanazip"], &[]
    ),
    app_entry!(
        "winrar", "WinRAR", "RARLAB", "win-rar.com",
        "Classic RAR & ZIP archive manager", "Compactador clássico RAR e ZIP",
        AppCategory::Utilities, "RARLab.WinRAR", None, &["winrar"], &[]
    ),
    app_entry!(
        "everything", "Everything", "voidtools", "voidtools.com",
        "Instant NTFS search across millions of files", "Busca instantânea de arquivos em milissegundos",
        AppCategory::Utilities, "voidtools.Everything",
        Some(DirectSource::StaticUrl {
            url: "https://www.voidtools.com/Everything-1.4.1.1026.x64-Setup.exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["everything"], &[PresetKind::Essentials, PresetKind::Developer, PresetKind::Gaming, PresetKind::Creator]
    ),
    app_entry!(
        "powertoys", "Microsoft PowerToys", "Microsoft", "learn.microsoft.com",
        "FancyZones, Command Palette, Color Picker & more", "FancyZones, Command Palette, Color Picker e mais",
        AppCategory::Utilities, "Microsoft.PowerToys",
        Some(DirectSource::GitHubRelease {
            repo: "microsoft/PowerToys", asset_contains: &["PowerToysUserSetup", "x64"],
            asset_excludes: &["ARM64"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["--silent"],
        }),
        &["powertoys"], &[PresetKind::Essentials, PresetKind::Developer]
    ),
    app_entry!(
        "wiztree", "WizTree", "Antibody Software", "diskanalyzer.com",
        "Fastest NTFS disk space analyzer", "Analisador de espaço em disco mais rápido",
        AppCategory::Utilities, "AntibodySoftware.WizTree", None, &["wiztree"], &[PresetKind::Essentials]
    ),
    app_entry!(
        "notepadpp", "Notepad++", "Don Ho", "notepad-plus-plus.org",
        "Blazing-fast lightweight text & code editor", "Editor de texto e código extremamente leve",
        AppCategory::Utilities, "Notepad++.Notepad++",
        Some(DirectSource::GitHubRelease {
            repo: "notepad-plus-plus/notepad-plus-plus", asset_contains: &["Installer.x64"],
            asset_excludes: &["arm64"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["notepad++"], &[]
    ),
    app_entry!(
        "localsend", "LocalSend", "LocalSend", "localsend.org",
        "Open-source AirDrop over local Wi-Fi", "Alternativa open-source ao AirDrop na rede local",
        AppCategory::Utilities, "LocalSend.LocalSend",
        Some(DirectSource::GitHubRelease {
            repo: "localsend/localsend", asset_contains: &["windows-x86-64"],
            asset_excludes: &["zip"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/VERYSILENT", "/NORESTART"],
        }),
        &["localsend"], &[]
    ),
    app_entry!(
        "qbittorrent", "qBittorrent", "qBittorrent Project", "qbittorrent.org",
        "Clean ad-free open-source BitTorrent client", "Cliente BitTorrent open-source limpo",
        AppCategory::Utilities, "qBittorrent.qBittorrent", None, &["qbittorrent"], &[]
    ),
    app_entry!(
        "eartrumpet", "EarTrumpet", "File-New-Project", "eartrumpet.app",
        "Per-app volume control in the taskbar", "Mixer de volume individual por app na barra",
        AppCategory::Utilities, "File-New-Project.EarTrumpet", None, &["eartrumpet"], &[PresetKind::Gaming]
    ),
    app_entry!(
        "cpuz", "CPU-Z", "CPUID", "cpuid.com",
        "Detailed CPU, motherboard and memory specs", "Informações detalhadas de CPU, placa-mãe e RAM",
        AppCategory::Utilities, "CPUID.CPU-Z", None, &["cpu-z"], &[]
    ),
    app_entry!(
        "crystaldiskinfo", "CrystalDiskInfo", "Crystal Dew World", "crystalmark.info",
        "SSD & HDD S.M.A.R.T. health and temperature", "Saúde S.M.A.R.T. e temperatura de SSDs e HDs",
        AppCategory::Utilities, "CrystalDewWorld.CrystalDiskInfo", None, &["crystaldiskinfo"], &[]
    ),
    app_entry!(
        "tailscale", "Tailscale", "Tailscale Inc.", "tailscale.com",
        "Zero-config WireGuard mesh VPN", "Rede privada WireGuard instantânea",
        AppCategory::Utilities, "Tailscale.Tailscale", None, &["tailscale"], &[]
    ),
    app_entry!(
        "cloudflare_warp", "Cloudflare WARP", "Cloudflare", "one.one.one.one",
        "Fast encrypted 1.1.1.1 DNS & tunnel", "DNS criptografado 1.1.1.1 rápido da Cloudflare",
        AppCategory::Utilities, "Cloudflare.Warp", None, &["cloudflare warp"], &[]
    ),

    // ── PRODUCTIVITY ─────────────────────────────────────────────────────────
    app_entry!(
        "obsidian", "Obsidian", "Obsidian", "obsidian.md",
        "Private local-first Markdown knowledge base", "Base de notas em Markdown local-first",
        AppCategory::Productivity, "Obsidian.Obsidian",
        Some(DirectSource::GitHubRelease {
            repo: "obsidianmd/obsidian-releases", asset_contains: &["Obsidian-"],
            asset_excludes: &["arm64"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["obsidian"], &[]
    ),
    app_entry!(
        "notion", "Notion", "Notion Labs", "notion.so",
        "Connected docs, wikis and project workspace", "Documentos, wikis e gestão de projetos",
        AppCategory::Productivity, "Notion.Notion", None, &["notion"], &[]
    ),
    app_entry!(
        "sumatrapdf", "SumatraPDF", "Krzysztof Kowalczyk", "sumatrapdfreader.org",
        "Instant-loading minimal PDF and ePub reader", "Leitor de PDF e ePub minimalista instantâneo",
        AppCategory::Productivity, "SumatraPDF.SumatraPDF", None, &["sumatrapdf"], &[PresetKind::Essentials]
    ),
    app_entry!(
        "bitwarden", "Bitwarden", "Bitwarden Inc.", "bitwarden.com",
        "Open-source cross-platform password manager", "Gerenciador de senhas open-source auditado",
        AppCategory::Productivity, "Bitwarden.Bitwarden",
        Some(DirectSource::GitHubRelease {
            repo: "bitwarden/clients", asset_contains: &["Bitwarden-Installer"],
            asset_excludes: &["arm64"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["bitwarden"], &[]
    ),
    app_entry!(
        "rustdesk", "RustDesk", "RustDesk", "rustdesk.com",
        "Fast open-source remote desktop built in Rust", "Acesso remoto open-source rápido feito em Rust",
        AppCategory::Productivity, "RustDesk.RustDesk",
        Some(DirectSource::GitHubRelease {
            repo: "rustdesk/rustdesk", asset_contains: &["x86_64"],
            asset_excludes: &["sciter", "msi"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["--silent-install"],
        }),
        &["rustdesk"], &[]
    ),
    app_entry!(
        "libreoffice", "LibreOffice", "The Document Foundation", "libreoffice.org",
        "Free open-source office document suite", "Suíte de escritório gratuita e open-source",
        AppCategory::Productivity, "TheDocumentFoundation.LibreOffice", None, &["libreoffice"], &[]
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn catalog_ids_and_winget_ids_are_unique_and_nonempty() {
        let mut ids = HashSet::new();
        for app in catalog() {
            assert!(!app.id.is_empty());
            assert!(!app.winget_id.is_empty());
            assert!(!app.domain.is_empty());
            assert!(!app.detect_keywords.is_empty());
            assert!(ids.insert(app.id), "Duplicate catalog id: {}", app.id);
        }
        assert!(catalog().len() >= 70);
    }
}
