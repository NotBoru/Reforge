#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppCategory {
    All,
    Ai,
    Dependencies,
    Fixes,
    Gaming,
    Browsers,
    Communication,
    Development,
    Media,
    Utilities,
    Productivity,
}

impl AppCategory {
    pub const ALL: &'static [AppCategory] = &[
        AppCategory::All,
        AppCategory::Ai,
        AppCategory::Dependencies,
        AppCategory::Fixes,
        AppCategory::Gaming,
        AppCategory::Browsers,
        AppCategory::Communication,
        AppCategory::Development,
        AppCategory::Media,
        AppCategory::Utilities,
        AppCategory::Productivity,
    ];
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetKind {
    Essentials,
    Ai,
    Dependencies,
    Gaming,
    Developer,
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
    OpenUrl {
        url: &'static str,
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
    #[allow(dead_code)]
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

macro_rules! app {
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
    // AI & coding tools
    app!(
        "claude_desktop", "Claude", "Anthropic", "claude.ai",
        "Official Anthropic Claude desktop assistant", "Aplicativo desktop oficial do Claude da Anthropic",
        AppCategory::Ai, "Anthropic.Claude", None,
        &["claude"], &[PresetKind::Ai]
    ),
    app!(
        "claude_code", "Claude Code CLI", "Anthropic", "claude.ai",
        "Agentic terminal coding assistant by Anthropic", "Agente de programação em terminal oficial da Anthropic",
        AppCategory::Ai, "Anthropic.ClaudeCode", None,
        &["claude code"], &[PresetKind::Ai, PresetKind::Developer]
    ),
    app!(
        "chatgpt", "ChatGPT", "OpenAI", "chatgpt.com",
        "Official OpenAI ChatGPT desktop app for Windows", "Aplicativo desktop oficial do ChatGPT para Windows",
        AppCategory::Ai, "OpenAI.ChatGPT", None,
        &["chatgpt"], &[PresetKind::Ai]
    ),
    app!(
        "codex_cli", "OpenAI Codex CLI", "OpenAI", "openai.com",
        "OpenAI terminal coding agent CLI", "Agente de programação CLI oficial da OpenAI",
        AppCategory::Ai, "OpenAI.Codex", None,
        &["codex cli"], &[PresetKind::Ai, PresetKind::Developer]
    ),
    app!(
        "gemini_desktop", "Google Gemini", "Google", "gemini.google.com",
        "Google Gemini AI desktop workspace", "Aplicativo desktop do Google Gemini AI",
        AppCategory::Ai, "Google.GeminiDesktop", None,
        &["gemini"], &[PresetKind::Ai]
    ),
    app!(
        "antigravity_ide", "Antigravity IDE", "Google", "antigravity.google",
        "Agent-first AI development environment by Google", "Ambiente de desenvolvimento com agentes de IA do Google",
        AppCategory::Ai, "Google.AntigravityIDE", None,
        &["antigravity ide"], &[PresetKind::Ai, PresetKind::Developer]
    ),
    app!(
        "antigravity_cli", "Antigravity CLI", "Google", "antigravity.google",
        "Antigravity agentic command-line interface", "Interface de linha de comando agêntica Antigravity",
        AppCategory::Ai, "Google.AntigravityCLI", None,
        &["antigravity cli"], &[]
    ),
    app!(
        "opencode_desktop", "OpenCode", "Anomaly", "opencode.ai",
        "Open-source AI coding desktop client", "Cliente desktop open-source de programação com IA",
        AppCategory::Ai, "Anomaly.OpenCode.Desktop", None,
        &["opencode"], &[PresetKind::Ai]
    ),
    app!(
        "opencode_cli", "OpenCode CLI", "Anomaly", "opencode.ai",
        "Terminal coding agent", "Agente de código no terminal",
        AppCategory::Ai, "Anomaly.OpenCode", None,
        &["opencode cli"], &[PresetKind::Ai, PresetKind::Developer]
    ),
    app!(
        "ivy_iii", "Ivy (III)", "III", "iii.rocks",
        "Desktop coding agent built on OpenCode", "Agente de código desktop baseado no OpenCode",
        AppCategory::Ai, "III.Ivy",
        Some(DirectSource::StaticUrl {
            url: "https://iii.rocks/downloads/III-Setup-1.0.17-x64.exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["ivy", "iii"], &[PresetKind::Ai]
    ),
    app!(
        "windsurf", "Windsurf", "Codeium", "windsurf.com",
        "Agentic AI code editor powered by Cascade", "Editor de código com agente IA Cascade integrado",
        AppCategory::Ai, "Codeium.Windsurf", None,
        &["windsurf"], &[]
    ),
    app!(
        "ollama", "Ollama", "Ollama", "ollama.com",
        "Run Llama, Qwen, DeepSeek & Gemma locally on GPU", "Rode modelos Llama, Qwen, DeepSeek e Gemma localmente",
        AppCategory::Ai, "Ollama.Ollama", None,
        &["ollama"], &[PresetKind::Ai]
    ),
    app!(
        "lm_studio", "LM Studio", "Element Labs", "lmstudio.ai",
        "Discover, download and run local GGUF/MLX LLMs", "Baixe e rode LLMs locais offline com interface visual",
        AppCategory::Ai, "ElementLabs.LMStudio", None,
        &["lm studio"], &[PresetKind::Ai]
    ),
    app!(
        "jan_ai", "Jan AI", "Jan", "jan.ai",
        "Open-source local-first ChatGPT alternative", "Alternativa open-source offline ao ChatGPT",
        AppCategory::Ai, "Jan.Jan", None,
        &["jan"], &[]
    ),
    app!(
        "cherry_studio", "Cherry Studio", "CherryHQ", "cherry-ai.com",
        "Multi-provider AI desktop client (OpenAI, Claude, Gemini)", "Cliente desktop unificado para Claude, OpenAI e Gemini",
        AppCategory::Ai, "kangfenmao.CherryStudio", None,
        &["cherry studio"], &[]
    ),
    app!(
        "comfyui", "ComfyUI Desktop", "Comfy Org", "comfy.org",
        "Node-based Flux, Stable Diffusion & AI video studio", "Estúdio nodal para Flux, Stable Diffusion e vídeo IA",
        AppCategory::Ai, "Comfy.ComfyUI-Desktop", None,
        &["comfyui"], &[]
    ),
    app!(
        "msty", "Msty AI", "CloudStack", "msty.app",
        "Local & cloud AI chat with RAG knowledge stacks", "Chat IA local e nuvem com base de conhecimento RAG",
        AppCategory::Ai, "CloudStack.Msty", None,
        &["msty"], &[]
    ),
    app!(
        "perplexity_comet", "Perplexity Comet", "Perplexity", "perplexity.ai",
        "Answer engine and browser by Perplexity", "Navegador e buscador da Perplexity",
        AppCategory::Ai, "Perplexity.Comet", None,
        &["comet"], &[]
    ),

    // Dependencies & runtimes
    app!(
        "vcredist_2015_x64", "Visual C++ 2015–2022 (x64)", "Microsoft", "microsoft.com",
        "Essential C++ runtime required by most games & apps", "Runtime C++ essencial para jogos e programas 64-bit",
        AppCategory::Dependencies, "Microsoft.VCRedist.2015+.x64",
        Some(DirectSource::StaticUrl {
            url: "https://aka.ms/vs/17/release/vc_redist.x64.exe",
            kind: InstallerKind::Exe, silent_args: &["/install", "/quiet", "/norestart"],
        }),
        &["visual c++ 2015-2022 redistributable (x64)", "visual c++ 2015-2022"],
        &[PresetKind::Dependencies, PresetKind::Essentials, PresetKind::Gaming]
    ),
    app!(
        "vcredist_2015_x86", "Visual C++ 2015–2022 (x86)", "Microsoft", "microsoft.com",
        "32-bit C++ runtime required by legacy games & launchers", "Runtime C++ 32-bit exigido por jogos e launchers",
        AppCategory::Dependencies, "Microsoft.VCRedist.2015+.x86",
        Some(DirectSource::StaticUrl {
            url: "https://aka.ms/vs/17/release/vc_redist.x86.exe",
            kind: InstallerKind::Exe, silent_args: &["/install", "/quiet", "/norestart"],
        }),
        &["visual c++ 2015-2022 redistributable (x86)"],
        &[PresetKind::Dependencies, PresetKind::Gaming]
    ),
    app!(
        "vcredist_2013_x64", "Visual C++ 2013 (x64)", "Microsoft", "microsoft.com",
        "Fixes msvcp120.dll / msvcr120.dll missing errors", "Corrige erros de msvcp120.dll e msvcr120.dll",
        AppCategory::Dependencies, "Microsoft.VCRedist.2013.x64", None,
        &["visual c++ 2013 redistributable (x64)"], &[PresetKind::Dependencies]
    ),
    app!(
        "vcredist_2013_x86", "Visual C++ 2013 (x86)", "Microsoft", "microsoft.com",
        "32-bit VC++ 2013 libraries for older games", "Bibliotecas VC++ 2013 32-bit para jogos antigos",
        AppCategory::Dependencies, "Microsoft.VCRedist.2013.x86", None,
        &["visual c++ 2013 redistributable (x86)"], &[PresetKind::Dependencies]
    ),
    app!(
        "vcredist_2012_x64", "Visual C++ 2012 (x64)", "Microsoft", "microsoft.com",
        "Fixes msvcp110.dll / msvcr110.dll missing DLLs", "Corrige ausência de msvcp110.dll e msvcr110.dll",
        AppCategory::Dependencies, "Microsoft.VCRedist.2012.x64", None,
        &["visual c++ 2012 redistributable (x64)"], &[PresetKind::Dependencies]
    ),
    app!(
        "vcredist_2010_x64", "Visual C++ 2010 (x64)", "Microsoft", "microsoft.com",
        "Fixes msvcp100.dll / msvcr100.dll in classic games", "Corrige msvcp100.dll e msvcr100.dll em jogos clássicos",
        AppCategory::Dependencies, "Microsoft.VCRedist.2010.x64", None,
        &["visual c++ 2010  x64"], &[PresetKind::Dependencies]
    ),
    app!(
        "directx_runtime", "DirectX End-User Runtime", "Microsoft", "microsoft.com",
        "Installs DX9/DX10/DX11 DLLs (fixes d3dx9_43.dll, xinput)", "Instala DLLs DX9/10/11 (corrige d3dx9_43.dll e xinput)",
        AppCategory::Dependencies, "Microsoft.DirectX", None,
        &["directx"], &[PresetKind::Dependencies, PresetKind::Gaming]
    ),
    app!(
        "dotnet_desktop_8", ".NET 8 Desktop Runtime", "Microsoft", "dotnet.microsoft.com",
        "LTS runtime for modern Windows desktop utilities", "Runtime .NET 8 LTS para utilitários modernos do Windows",
        AppCategory::Dependencies, "Microsoft.DotNet.DesktopRuntime.8", None,
        &[".net windows desktop runtime - 8."], &[PresetKind::Dependencies, PresetKind::Essentials]
    ),
    app!(
        "dotnet_desktop_9", ".NET 9 Desktop Runtime", "Microsoft", "dotnet.microsoft.com",
        "Latest .NET 9 desktop framework runtime", "Runtime mais recente .NET 9 Desktop",
        AppCategory::Dependencies, "Microsoft.DotNet.DesktopRuntime.9", None,
        &[".net windows desktop runtime - 9."], &[PresetKind::Dependencies]
    ),
    app!(
        "dotnet_desktop_6", ".NET 6 Desktop Runtime", "Microsoft", "dotnet.microsoft.com",
        "Legacy .NET 6 runtime for compatibility", "Runtime .NET 6 para compatibilidade com apps antigos",
        AppCategory::Dependencies, "Microsoft.DotNet.DesktopRuntime.6", None,
        &[".net windows desktop runtime - 6."], &[]
    ),
    app!(
        "temurin_21", "Java 21 LTS (Temurin JDK)", "Eclipse Adoptium", "adoptium.net",
        "Fast open-source Java 21 runtime (Minecraft 1.21+)", "Java 21 LTS rápido para Minecraft 1.21+ e dev",
        AppCategory::Dependencies, "EclipseAdoptium.Temurin.21.JDK", None,
        &["temurin jdk with hotspot 21"], &[PresetKind::Dependencies, PresetKind::Gaming]
    ),
    app!(
        "temurin_17", "Java 17 LTS (Temurin JDK)", "Eclipse Adoptium", "adoptium.net",
        "Java 17 runtime for Minecraft 1.18–1.20 modpacks", "Java 17 para modpacks de Minecraft 1.18–1.20",
        AppCategory::Dependencies, "EclipseAdoptium.Temurin.17.JDK", None,
        &["temurin jdk with hotspot 17"], &[]
    ),
    app!(
        "temurin_8", "Java 8 (Temurin JRE)", "Eclipse Adoptium", "adoptium.net",
        "Java 8 runtime for Minecraft 1.8.9 PvP & legacy mods", "Java 8 para Minecraft 1.8.9 PvP e mods clássicos",
        AppCategory::Dependencies, "EclipseAdoptium.Temurin.8.JRE", None,
        &["temurin jre with hotspot 8"], &[]
    ),
    app!(
        "openal", "OpenAL Audio Runtime", "Creative Labs", "openal.org",
        "3D audio library required by older PC games", "Biblioteca de áudio 3D exigida por jogos antigos",
        AppCategory::Dependencies, "CreativeTechnology.OpenAL", None,
        &["openal"], &[PresetKind::Dependencies]
    ),
    app!(
        "webview2", "Edge WebView2 Runtime", "Microsoft", "microsoft.com",
        "WebView2 engine for stripped Windows ISOs (LTSC/Tiny11)", "Runtime WebView2 para ISOs enxutas (LTSC/Tiny11)",
        AppCategory::Dependencies, "Microsoft.EdgeWebView2Runtime",
        Some(DirectSource::StaticUrl {
            url: "https://go.microsoft.com/fwlink/p/?LinkId=2124703",
            kind: InstallerKind::Exe, silent_args: &["/silent", "/install"],
        }),
        &["webview2 runtime"], &[]
    ),
    app!(
        "xna_framework", "Microsoft XNA Framework 4.0", "Microsoft", "microsoft.com",
        "Runtime required by classic indie games", "Runtime exigido por jogos indie clássicos",
        AppCategory::Dependencies, "Microsoft.XNARedist", None,
        &["xna framework"], &[]
    ),
    app!(
        "eac_eos", "Easy Anti-Cheat (EOS)", "Epic Games", "easy.ac",
        "Epic Online Services and Easy Anti-Cheat runtime", "Runtime do Epic Online Services e Easy Anti-Cheat",
        AppCategory::Dependencies, "EpicGames.EpicOnlineServices", None,
        &["epic online services", "easy anti-cheat"], &[PresetKind::Dependencies, PresetKind::Gaming]
    ),

    // Fixes, drivers & hardware tools
    app!(
        "ddu", "Display Driver Uninstaller (DDU)", "Wagnardsoft", "wagnardsoft.com",
        "Cleanly wipe NVIDIA, AMD & Intel GPU drivers without traces", "Remove drivers NVIDIA, AMD e Intel sem rastros",
        AppCategory::Fixes, "Wagnardsoft.DisplayDriverUninstaller", None,
        &["display driver uninstaller"], &[]
    ),
    app!(
        "nvcleanstall", "NVCleanstall", "TechPowerUp", "techpowerup.com",
        "Install minimal bloat-free NVIDIA drivers (zero telemetry)", "Instala drivers NVIDIA limpos sem telemetria ou bloat",
        AppCategory::Fixes, "TechPowerUp.NVCleanstall", None,
        &["nvcleanstall"], &[]
    ),
    app!(
        "nvidia_app", "NVIDIA App", "NVIDIA", "nvidia.com",
        "NVIDIA drivers and ShadowPlay", "Drivers NVIDIA e ShadowPlay",
        AppCategory::Fixes, "Nvidia.NvidiaApp", None,
        &["nvidia app"], &[]
    ),
    app!(
        "amd_adrenalin", "AMD Software: Adrenalin & Chipset", "AMD", "amd.com",
        "Official AMD GPU and Ryzen chipset driver installer", "Instalador oficial de drivers AMD Radeon e Chipset Ryzen",
        AppCategory::Fixes, "AMD.Adrenalin",
        Some(DirectSource::OpenUrl {
            url: "https://www.amd.com/en/support/download/drivers.html",
        }),
        &["amd software", "amd chipset"], &[]
    ),
    app!(
        "amd_ryzen_master", "AMD Ryzen Master", "AMD", "amd.com",
        "CPU tuning and monitoring utility for Ryzen", "Monitoramento e ajustes para processadores AMD Ryzen",
        AppCategory::Fixes, "AMD.RyzenMaster",
        Some(DirectSource::OpenUrl {
            url: "https://www.amd.com/en/products/software/ryzen-master.html",
        }),
        &["ryzen master"], &[]
    ),
    app!(
        "amd_ocat", "AMD OCAT", "AMD", "amd.com",
        "Frame-time overlay and GPU performance capture", "Overlay de frametime e captura de desempenho AMD",
        AppCategory::Fixes, "AMD.OCAT", None,
        &["ocat"], &[]
    ),
    app!(
        "process_lasso", "Process Lasso", "Bitsum", "bitsum.com",
        "CPU affinity, ProBalance anti-stutter & power profiles", "Afinidade de CPU, ProBalance anti-stutter e prioridade",
        AppCategory::Fixes, "Bitsum.ProcessLasso", None,
        &["process lasso"], &[]
    ),
    app!(
        "latencymon", "LatencyMon", "Resplendence", "resplendence.com",
        "Diagnose DPC/ISR audio & gaming micro-stutter drivers", "Diagnostica drivers causando DPC latency e stutters",
        AppCategory::Fixes, "Resplendence.LatencyMon", None,
        &["latencymon"], &[]
    ),
    app!(
        "hwinfo", "HWiNFO64", "REALiX", "hwinfo.com",
        "Precision CPU/GPU temperature, voltage & sensor diagnostics", "Sensores precisos de temperatura, voltagem e clock",
        AppCategory::Fixes, "REALiX.HWiNFO", None,
        &["hwinfo"], &[PresetKind::Gaming]
    ),
    app!(
        "hwmonitor", "HWMonitor", "CPUID", "cpuid.com",
        "Simple real-time hardware temperature & fan monitor", "Monitor simples de temperaturas e ventoinhas",
        AppCategory::Fixes, "CPUID.HWMonitor", None,
        &["hwmonitor"], &[]
    ),
    app!(
        "cpuz", "CPU-Z", "CPUID", "cpuid.com",
        "Detailed CPU, motherboard & RAM timings inspection", "Especificações completas de CPU, placa-mãe e RAM",
        AppCategory::Fixes, "CPUID.CPU-Z", None,
        &["cpu-z"], &[]
    ),
    app!(
        "gpuz", "GPU-Z", "TechPowerUp", "techpowerup.com",
        "Graphics card specs, VBIOS, VRAM & PCIe link checker", "Informações completas de GPU, VRAM, PCIe e sensores",
        AppCategory::Fixes, "TechPowerUp.GPU-Z", None,
        &["gpu-z"], &[]
    ),
    app!(
        "crystaldiskinfo", "CrystalDiskInfo", "Crystal Dew World", "crystalmark.info",
        "NVMe/SSD/HDD S.M.A.R.T. health & wear percentage", "Saúde S.M.A.R.T., desgaste e temperatura de SSDs",
        AppCategory::Fixes, "CrystalDewWorld.CrystalDiskInfo", None,
        &["crystaldiskinfo"], &[]
    ),
    app!(
        "crystaldiskmark", "CrystalDiskMark", "Crystal Dew World", "crystalmark.info",
        "Sequential & random 4K SSD read/write benchmark", "Benchmark de leitura e escrita para SSDs e NVMe",
        AppCategory::Fixes, "CrystalDewWorld.CrystalDiskMark", None,
        &["crystaldiskmark"], &[]
    ),
    app!(
        "occt", "OCCT", "OCBASE", "ocbase.com",
        "CPU, GPU, RAM & PSU stability and thermal stress tester", "Teste de estabilidade para CPU, GPU, RAM e fonte",
        AppCategory::Fixes, "OCBase.OCCT.Personal", None,
        &["occt"], &[]
    ),
    app!(
        "fancontrol", "FanControl", "Rem0o", "getfancontrol.com",
        "Lightweight custom GPU & case fan curve control", "Controle leve e avançado de curvas de ventoinhas",
        AppCategory::Fixes, "Rem0o.FanControl", None,
        &["fancontrol"], &[]
    ),
    app!(
        "ds4windows", "DS4Windows", "Ryochan7", "ds4-windows.com",
        "Use PlayStation DualSense/DS4 controllers in any PC game", "Use controles PS4/DualSense nativamente em qualquer jogo",
        AppCategory::Fixes, "Ryochan7.DS4Windows", None,
        &["ds4windows"], &[]
    ),
    app!(
        "logi_options_plus", "Logi Options+", "Logitech", "logitech.com",
        "Custom DPI, buttons & Flow for Logitech MX mice & keyboards", "Ajuste DPI, teclas e Flow para mouses e teclados Logitech",
        AppCategory::Fixes, "Logitech.OptionsPlus", None,
        &["logi options+"], &[]
    ),
    app!(
        "razer_synapse4", "Razer Synapse 4", "Razer", "razer.com",
        "Driver & macro suite for Razer mice, keyboards & headsets", "Drivers, DPI e macros para periféricos Razer",
        AppCategory::Fixes, "RazerInc.RazerInstaller.Synapse4", None,
        &["razer synapse"], &[]
    ),
    app!(
        "steelseries_gg", "SteelSeries GG", "SteelSeries", "steelseries.com",
        "Engine, Sonar spatial audio EQ & Moments for peripherals", "Drivers Engine e equalizador espacial Sonar para periféricos",
        AppCategory::Fixes, "SteelSeries.GG", None,
        &["steelseries gg"], &[]
    ),
    app!(
        "corsair_icue5", "Corsair iCUE 5", "Corsair", "corsair.com",
        "Control Corsair mice, keyboards, coolers, fans & RGB", "Controle mouses, teclados, water coolers e RGB Corsair",
        AppCategory::Fixes, "Corsair.iCUE.5", None,
        &["corsair icue"], &[]
    ),
    app!(
        "signalrgb", "SignalRGB", "WhirlwindFX", "signalrgb.com",
        "Unified RGB & fan sync across all PC brands", "Sincronize iluminação RGB e fans de todas as marcas",
        AppCategory::Fixes, "WhirlwindFX.SignalRgb", None,
        &["signalrgb"], &[]
    ),
    app!(
        "g_helper", "G-Helper (ASUS Control)", "Serge", "github.com",
        "Ultra-lightweight Armoury Crate replacement for ASUS laptops", "Substituto ultraleve do Armoury Crate para notebooks ASUS",
        AppCategory::Fixes, "seerge.g-helper", None,
        &["g-helper"], &[]
    ),
    app!(
        "intel_dsa", "Intel Driver & Support Assistant", "Intel", "intel.com",
        "Official automatic Intel Wi-Fi, Bluetooth & GPU driver updater", "Atualizador oficial de drivers Intel Wi-Fi, Bluetooth e GPU",
        AppCategory::Fixes, "Intel.IntelDriverAndSupportAssistant", None,
        &["intel driver"], &[]
    ),
    app!(
        "voicemeeter_banana", "Voicemeeter Banana", "VB-Audio", "vb-audio.com",
        "Virtual multi-bus audio mixer for microphones & streaming", "Mixer virtual de áudio para microfones, Discord e lives",
        AppCategory::Fixes, "VB-Audio.Voicemeeter.Banana", None,
        &["voicemeeter"], &[]
    ),
    app!(
        "elgato_streamdeck", "Elgato Stream Deck", "Elgato", "elgato.com",
        "Macro pad software & plugin ecosystem for Elgato decks", "Software de atalhos e macros para Elgato Stream Deck",
        AppCategory::Fixes, "Elgato.StreamDeck", None,
        &["stream deck"], &[]
    ),
    app!(
        "vigembus", "ViGEmBus Controller Driver", "Nefarius", "github.com",
        "Virtual Xbox 360/DS4 gamepad driver fix for PC controllers", "Driver virtual de controle essencial para DS4Windows/Parsec",
        AppCategory::Fixes, "Nefarius.ViGEmBus", None,
        &["vigembus"], &[]
    ),
    app!(
        "equalizer_apo", "Equalizer APO", "Jonas Thedering", "sourceforge.net",
        "Zero-latency system-wide parametric EQ & mic filters", "Equalizador paramétrico sem latência e filtros de microfone",
        AppCategory::Fixes, "EqualizerAPO.EqualizerAPO", None,
        &["equalizer apo"], &[]
    ),
    app!(
        "vb_cable", "VB-Cable Virtual Audio", "VB-Audio", "vb-audio.com",
        "Virtual audio routing cable for OBS, Discord & streaming", "Cabo de áudio virtual para rotear som no OBS e Discord",
        AppCategory::Fixes, "VB-Audio.Cable", None,
        &["vb-cable"], &[]
    ),
    app!(
        "openrgb", "OpenRGB", "CalcProgrammer1", "openrgb.org",
        "Control all RGB hardware without bloated vendor software", "Controle RGB unificado sem softwares pesados de fabricante",
        AppCategory::Fixes, "OpenRGB.OpenRGB", None,
        &["openrgb"], &[]
    ),
    app!(
        "autohotkey", "AutoHotkey v2", "AutoHotkey Foundation", "autohotkey.com",
        "Fast keyboard macros, key remapping & automation", "Macros de teclado, remapeamento e automação leve",
        AppCategory::Fixes, "AutoHotkey.AutoHotkey", None,
        &["autohotkey"], &[]
    ),

    // Games, launchers & clients
    app!(
        "hydra", "Hydra Launcher", "Hydra", "hydralauncher.gg",
        "Open-source game launcher with library, achievements & cloud saves", "Launcher open-source com biblioteca, conquistas e nuvem",
        AppCategory::Gaming, "HydraLauncher.Hydra",
        Some(DirectSource::GitHubRelease {
            repo: "hydralauncher/hydra", asset_contains: &["setup"],
            asset_excludes: &["arm64", "blockmap"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["hydra"], &[PresetKind::Gaming]
    ),
    app!(
        "lunar_client", "Lunar Client", "Moonsworth", "lunarclient.com",
        "High-FPS Minecraft PvP & modpack client by Moonsworth", "Cliente de alto FPS para Minecraft da Moonsworth",
        AppCategory::Gaming, "Moonsworth.LunarClient",
        Some(DirectSource::StaticUrl {
            url: "https://launcherupdates.lunarclientcdn.com/Lunar%20Client%20v3.3.1.exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["lunar client"], &[]
    ),
    app!(
        "cheatbreaker", "CheatBreaker", "CheatBreaker", "cheatbreaker.net",
        "FPS-boosted classic Minecraft 1.7.10 / 1.8.9 PvP client", "Cliente clássico focado em FPS máximo para Minecraft PvP",
        AppCategory::Gaming, "CheatBreaker.CheatBreaker",
        Some(DirectSource::StaticUrl {
            url: "https://cdn.cheatbreaker.net/launcher/CheatBreaker-Setup.exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["cheatbreaker"], &[]
    ),
    app!(
        "badlion", "Badlion Client", "Moonsworth / Badlion", "client.badlion.net",
        "Minecraft PvP client with built-in mods", "Cliente de Minecraft com mods embutidos",
        AppCategory::Gaming, "Badlion.BadlionClient", None,
        &["badlion"], &[]
    ),
    app!(
        "minecraft_launcher", "Minecraft Launcher", "Mojang / Microsoft", "minecraft.net",
        "Official Minecraft Java & Bedrock launcher", "Launcher oficial do Minecraft Java e Bedrock",
        AppCategory::Gaming, "Mojang.MinecraftLauncher", None,
        &["minecraft launcher"], &[]
    ),
    app!(
        "prism_launcher", "Prism Launcher", "Prism Launcher", "prismlauncher.org",
        "Open-source multi-instance Minecraft launcher", "Launcher open-source multi-instância para Minecraft",
        AppCategory::Gaming, "PrismLauncher.PrismLauncher",
        Some(DirectSource::GitHubRelease {
            repo: "PrismLauncher/PrismLauncher", asset_contains: &["Windows-MSVC-Setup"],
            asset_excludes: &["arm64", "Portable"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/VERYSILENT", "/NORESTART"],
        }),
        &["prism launcher"], &[]
    ),
    app!(
        "modrinth", "Modrinth App", "Modrinth", "modrinth.com",
        "Clean modern Minecraft mod & modpack manager", "Gerenciador moderno de mods e modpacks do Modrinth",
        AppCategory::Gaming, "Modrinth.ModrinthApp", None,
        &["modrinth"], &[]
    ),
    app!(
        "curseforge", "CurseForge", "Overwolf", "curseforge.com",
        "Modpack manager for Minecraft, WoW, Sims & more", "Gerenciador de modpacks para Minecraft, WoW e mais",
        AppCategory::Gaming, "Overwolf.CurseForge", None,
        &["curseforge"], &[]
    ),
    app!(
        "steam", "Steam", "Valve", "store.steampowered.com",
        "PC gaming platform, store and library", "Principal plataforma e biblioteca de jogos para PC",
        AppCategory::Gaming, "Valve.Steam",
        Some(DirectSource::StaticUrl {
            url: "https://cdn.fastly.steamstatic.com/client/installer/SteamSetup.exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["steam"], &[PresetKind::Gaming]
    ),
    app!(
        "epic", "Epic Games Launcher", "Epic Games", "epicgames.com",
        "Epic Games Store, Unreal Engine & weekly free titles", "Loja Epic Games, Unreal Engine e jogos grátis semanais",
        AppCategory::Gaming, "EpicGames.EpicGamesLauncher",
        Some(DirectSource::StaticUrl {
            url: "https://launcher-public-service-prod06.ol.epicgames.com/launcher/api/installer/download/EpicGamesLauncherInstaller.msi",
            kind: InstallerKind::Msi, silent_args: &["/qn", "/norestart"],
        }),
        &["epic games launcher"], &[]
    ),
    app!(
        "riot_client", "Riot Client & Vanguard", "Riot Games", "riotgames.com",
        "Riot Games launcher and Vanguard anti-cheat", "Launcher da Riot Games e anti-cheat Vanguard",
        AppCategory::Gaming, "RiotGames.Valorant.Live", None,
        &["riot client", "riot vanguard", "valorant"], &[]
    ),
    app!(
        "ldplayer", "LDPlayer 9", "XuanZhi", "ldplayer.net",
        "Android emulator for PC games", "Emulador Android para jogos no PC",
        AppCategory::Gaming, "XuanZhi.LDPlayer9",
        Some(DirectSource::StaticUrl {
            url: "https://res.ldrescdn.com/download/package/LDPlayer9.0.exe?n=LDPlayer9.0_ens_1001_ld.exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["ldplayer"], &[]
    ),
    app!(
        "bluestacks", "BlueStacks", "BlueStack Systems", "bluestacks.com",
        "Android app player for Windows", "Emulador de apps e jogos Android para Windows",
        AppCategory::Gaming, "BlueStack.BlueStacks", None,
        &["bluestacks"], &[]
    ),
    app!(
        "battlenet", "Battle.net", "Blizzard", "battle.net",
        "Blizzard launcher for Overwatch 2, WoW & Diablo", "Launcher oficial da Blizzard",
        AppCategory::Gaming, "Blizzard.BattleNet", None,
        &["battle.net"], &[]
    ),
    app!(
        "ea_app", "EA App", "Electronic Arts", "ea.com",
        "Electronic Arts PC storefront and game client", "Plataforma oficial de jogos da EA",
        AppCategory::Gaming, "ElectronicArts.EADesktop", None,
        &["ea app"], &[]
    ),
    app!(
        "ubisoft_connect", "Ubisoft Connect", "Ubisoft", "ubisoft.com",
        "Ubisoft PC client for Rainbow Six, Assassin's Creed & more", "Cliente oficial Ubisoft para PC",
        AppCategory::Gaming, "Ubisoft.Connect", None,
        &["ubisoft connect"], &[]
    ),
    app!(
        "gog_galaxy", "GOG Galaxy", "GOG", "gog.com",
        "DRM-free game store and unified launcher", "Loja sem DRM e biblioteca unificada GOG",
        AppCategory::Gaming, "GOG.Galaxy", None,
        &["gog galaxy"], &[]
    ),
    app!(
        "heroic", "Heroic Games Launcher", "Heroic", "heroicgameslauncher.com",
        "Fast open-source launcher for Epic, GOG & Amazon Games", "Launcher open-source rápido para Epic, GOG e Amazon",
        AppCategory::Gaming, "HeroicGamesLauncher.HeroicGamesLauncher", None,
        &["heroic"], &[]
    ),
    app!(
        "playnite", "Playnite", "Josef Nemec", "playnite.link",
        "Unified open-source library across all launchers & emulators", "Biblioteca unificada para todos launchers e emuladores",
        AppCategory::Gaming, "Playnite.Playnite",
        Some(DirectSource::GitHubRelease {
            repo: "JosefNemec/Playnite", asset_contains: &["Playnite"],
            asset_excludes: &["Portable"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/VERYSILENT", "/NORESTART"],
        }),
        &["playnite"], &[]
    ),
    app!(
        "roblox", "Roblox", "Roblox Corporation", "roblox.com",
        "Roblox desktop game player", "Cliente desktop do Roblox",
        AppCategory::Gaming, "Roblox.Roblox", None,
        &["roblox"], &[]
    ),
    app!(
        "msi_afterburner", "MSI Afterburner + RTSS", "MSI", "msi.com",
        "FPS overlay, frame-time graph, fan curve & GPU tuning", "Overlay de FPS, frametime, curvas de fan e overclock",
        AppCategory::Gaming, "Guru3D.Afterburner", None,
        &["msi afterburner"], &[PresetKind::Gaming]
    ),
    app!(
        "medal", "Medal.tv", "Medal", "medal.tv",
        "Instant game clipping and highlight sharing", "Clipes automáticos e highlights de jogos",
        AppCategory::Gaming, "Medal.Medal", None,
        &["medal"], &[]
    ),
    app!(
        "parsec", "Parsec", "Unity / Parsec", "parsec.app",
        "Ultra-low latency game streaming & remote desktop", "Streaming e acesso remoto com latência mínima",
        AppCategory::Gaming, "Parsec.Parsec",
        Some(DirectSource::StaticUrl {
            url: "https://builds.parsec.app/package/parsec-windows.exe",
            kind: InstallerKind::Exe, silent_args: &["/silent", "/shared"],
        }),
        &["parsec"], &[]
    ),
    app!(
        "moonlight", "Moonlight", "Moonlight Team", "moonlight-stream.org",
        "Open-source 120fps low-latency Sunshine/GameStream client", "Cliente open-source de streaming local em 120fps",
        AppCategory::Gaming, "MoonlightGameStreamingProject.Moonlight", None,
        &["moonlight"], &[]
    ),
    app!(
        "retroarch", "RetroArch", "Libretro", "retroarch.com",
        "Unified frontend for classic console emulators", "Interface unificada para emuladores clássicos",
        AppCategory::Gaming, "Libretro.RetroArch", None,
        &["retroarch"], &[]
    ),
    app!(
        "pcsx2", "PCSX2", "PCSX2 Team", "pcsx2.net",
        "High-accuracy PlayStation 2 emulator", "Emulador de PlayStation 2 em alta resolução",
        AppCategory::Gaming, "PCSX2Team.PCSX2", None,
        &["pcsx2"], &[]
    ),
    app!(
        "rpcs3", "RPCS3", "RPCS3 Team", "rpcs3.net",
        "Open-source PlayStation 3 emulator", "Emulador open-source de PlayStation 3",
        AppCategory::Gaming, "RPCS3.RPCS3", None,
        &["rpcs3"], &[]
    ),
    app!(
        "dolphin", "Dolphin Emulator", "Dolphin Team", "dolphin-emu.org",
        "GameCube & Wii emulator", "Emulador de GameCube e Nintendo Wii",
        AppCategory::Gaming, "DolphinEmulator.Dolphin", None,
        &["dolphin"], &[]
    ),
    app!(
        "logitech_ghub", "Logitech G HUB", "Logitech", "logitechg.com",
        "DPI, macros & lighting for Logitech G mice and headsets", "DPI, macros e perfis para periféricos Logitech G",
        AppCategory::Gaming, "Logitech.GHUB", None,
        &["logitech g hub"], &[]
    ),

    // Browsers
    app!(
        "chrome", "Google Chrome", "Google", "google.com/chrome",
        "Fast web browser with Google sync", "Navegador rápido com sincronização Google",
        AppCategory::Browsers, "Google.Chrome",
        Some(DirectSource::StaticUrl {
            url: "https://dl.google.com/dl/chrome/install/googlechromestandaloneenterprise64.msi",
            kind: InstallerKind::Msi, silent_args: &["/qn", "/norestart"],
        }),
        &["google chrome"], &[PresetKind::Essentials]
    ),
    app!(
        "brave", "Brave Browser", "Brave Software", "brave.com",
        "Privacy-first Chromium browser with built-in ad blocker", "Navegador Chromium privado com bloqueador nativo",
        AppCategory::Browsers, "Brave.Brave",
        Some(DirectSource::StaticUrl {
            url: "https://laptop-updates.brave.com/latest/winx64",
            kind: InstallerKind::Exe, silent_args: &["/silent", "/install"],
        }),
        &["brave"], &[PresetKind::Developer, PresetKind::Gaming]
    ),
    app!(
        "firefox", "Mozilla Firefox", "Mozilla", "firefox.com",
        "Open-source browser built for privacy and speed", "Navegador open-source focado em velocidade e privacidade",
        AppCategory::Browsers, "Mozilla.Firefox",
        Some(DirectSource::StaticUrl {
            url: "https://download.mozilla.org/?product=firefox-latest-ssl&os=win64&lang=en-US",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["mozilla firefox"], &[]
    ),
    app!(
        "zen", "Zen Browser", "Zen Team", "zen-browser.app",
        "Calm minimal Firefox browser with vertical tabs", "Navegador minimalista baseado no Firefox com abas verticais",
        AppCategory::Browsers, "Zen-Team.Zen-Browser",
        Some(DirectSource::GitHubRelease {
            repo: "zen-browser/desktop", asset_contains: &["installer"],
            asset_excludes: &["arm64"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["zen browser", "zen twilight"], &[]
    ),
    app!(
        "arc", "Arc Browser", "The Browser Company", "arc.net",
        "Modern workspace browser with Spaces and Split View", "Navegador moderno com Spaces e visão dividida",
        AppCategory::Browsers, "TheBrowserCompany.Arc", None, &["arc"], &[]
    ),
    app!(
        "opera_gx", "Opera GX", "Opera", "opera.com/gx",
        "Gaming browser with RAM, CPU & network limiters", "Navegador gamer com limitador de RAM, CPU e rede",
        AppCategory::Browsers, "Opera.OperaGX", None, &["opera gx"], &[]
    ),
    app!(
        "vivaldi", "Vivaldi", "Vivaldi Technologies", "vivaldi.com",
        "Power-user browser with deep tab customization", "Navegador avançado altamente personalizável",
        AppCategory::Browsers, "Vivaldi.Vivaldi", None, &["vivaldi"], &[]
    ),
    app!(
        "librewolf", "LibreWolf", "LibreWolf", "librewolf.net",
        "Hardened Firefox fork with zero telemetry", "Fork do Firefox focado em privacidade máxima",
        AppCategory::Browsers, "LibreWolf.LibreWolf", None, &["librewolf"], &[]
    ),
    app!(
        "floorp", "Floorp", "Ablaze", "floorp.app",
        "Fast Japanese Firefox fork with workspaces", "Fork rápido do Firefox com workspaces",
        AppCategory::Browsers, "Ablaze.Floorp", None, &["floorp"], &[]
    ),
    app!(
        "tor_browser", "Tor Browser", "Tor Project", "torproject.org",
        "Anonymous browsing over the Tor onion network", "Navegação anônima pela rede Tor",
        AppCategory::Browsers, "TorProject.TorBrowser", None, &["tor browser"], &[]
    ),

    // Communication
    app!(
        "discord", "Discord", "Discord Inc.", "discord.com",
        "Voice, video, and community chat", "Chat de voz, vídeo e comunidades",
        AppCategory::Communication, "Discord.Discord",
        Some(DirectSource::StaticUrl {
            url: "https://discord.com/api/downloads/distributions/app/installers/latest?channel=stable&platform=win&arch=x64",
            kind: InstallerKind::Exe, silent_args: &["-s"],
        }),
        &["discord"], &[PresetKind::Essentials, PresetKind::Gaming]
    ),
    app!(
        "vesktop", "Vesktop", "Vencord", "vencord.dev",
        "Lightweight Discord client with Vencord built in", "Cliente leve para Discord com Vencord embutido",
        AppCategory::Communication, "Vencord.Vesktop",
        Some(DirectSource::GitHubRelease {
            repo: "Vencord/Vesktop", asset_contains: &["Setup"],
            asset_excludes: &["arm64"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["vesktop"], &[]
    ),
    app!(
        "telegram", "Telegram Desktop", "Telegram", "telegram.org",
        "Fast, cloud-synced desktop messenger", "Mensageiro rápido sincronizado na nuvem",
        AppCategory::Communication, "Telegram.TelegramDesktop",
        Some(DirectSource::StaticUrl {
            url: "https://telegram.org/dl/desktop/win64",
            kind: InstallerKind::Exe, silent_args: &["/VERYSILENT", "/NORESTART"],
        }),
        &["telegram desktop"], &[]
    ),
    app!(
        "whatsapp", "WhatsApp", "Meta", "whatsapp.com",
        "Official WhatsApp desktop client", "Cliente oficial do WhatsApp para Windows",
        AppCategory::Communication, "9NKSQGP7F2NH", None, &["whatsapp"], &[]
    ),
    app!(
        "slack", "Slack", "Salesforce", "slack.com",
        "Team messaging and workspace hub", "Comunicação corporativa e canais de equipe",
        AppCategory::Communication, "SlackTechnologies.Slack", None, &["slack"], &[]
    ),
    app!(
        "zoom", "Zoom Workplace", "Zoom", "zoom.us",
        "Video conferencing and screen sharing", "Videoconferências e reuniões online",
        AppCategory::Communication, "Zoom.Zoom",
        Some(DirectSource::StaticUrl {
            url: "https://zoom.us/client/latest/ZoomInstallerFull.msi?archType=x64",
            kind: InstallerKind::Msi, silent_args: &["/qn", "/norestart"],
        }),
        &["zoom"], &[]
    ),
    app!(
        "teamspeak", "TeamSpeak 3", "TeamSpeak", "teamspeak.com",
        "Ultra-low latency voice chat for competitive gaming", "Chat de voz de latência baixíssima para jogos",
        AppCategory::Communication, "TeamSpeakSystems.TeamSpeakClient", None, &["teamspeak"], &[]
    ),
    app!(
        "signal", "Signal", "Signal Foundation", "signal.org",
        "End-to-end encrypted private messenger", "Mensageiro privado com criptografia ponta a ponta",
        AppCategory::Communication, "OpenWhisperSystems.Signal", None, &["signal"], &[]
    ),

    // Development
    app!(
        "vscode", "Visual Studio Code", "Microsoft", "code.visualstudio.com",
        "Code editor by Microsoft", "Editor de código da Microsoft",
        AppCategory::Development, "Microsoft.VisualStudioCode",
        Some(DirectSource::StaticUrl {
            url: "https://code.visualstudio.com/sha/download?build=stable&os=win32-x64-user",
            kind: InstallerKind::Exe,
            silent_args: &["/VERYSILENT", "/NORESTART", "/MERGETASKS=!runcode,addcontextmenufiles,addcontextmenufolders,addtopath"],
        }),
        &["visual studio code"], &[PresetKind::Developer]
    ),
    app!(
        "vs2022_community", "Visual Studio 2022", "Microsoft", "visualstudio.microsoft.com",
        "IDE for C++, C#, .NET and Windows apps (Community)", "IDE da Microsoft para C++, C# e .NET (Community)",
        AppCategory::Development, "Microsoft.VisualStudio.2022.Community", None,
        &["visual studio community 2022"], &[PresetKind::Developer]
    ),
    app!(
        "vs2022_buildtools", "Visual Studio Build Tools 2022", "Microsoft", "visualstudio.microsoft.com",
        "MSVC C/C++ compiler and Windows SDK toolchain", "Compilador MSVC C/C++ e Windows SDK sem a IDE",
        AppCategory::Development, "Microsoft.VisualStudio.2022.BuildTools", None,
        &["visual studio build tools"], &[PresetKind::Dependencies, PresetKind::Developer]
    ),
    app!(
        "retoolkit", "REToolkit", "Mente Binária", "mentebinaria.com.br",
        "Reverse engineering and binary analysis toolkit", "Pacote de ferramentas para engenharia reversa e análise",
        AppCategory::Development, "MenteBinaria.REToolkit",
        Some(DirectSource::StaticUrl {
            url: "https://github.com/mentebinaria/retoolkit/releases/download/2023.10/retoolkit_2023.10_setup.exe",
            kind: InstallerKind::Exe, silent_args: &["/VERYSILENT", "/NORESTART"],
        }),
        &["retoolkit"], &[]
    ),
    app!(
        "cursor", "Cursor", "Anysphere", "cursor.com",
        "Code editor with built-in AI assistance", "Editor de código com assistente de IA integrado",
        AppCategory::Ai, "Anysphere.Cursor", None, &["cursor"], &[PresetKind::Ai, PresetKind::Developer]
    ),
    app!(
        "zed", "Zed", "Zed Industries", "zed.dev",
        "GPU-accelerated code editor written in Rust", "Editor de código em Rust com aceleração por GPU",
        AppCategory::Development, "ZedIndustries.Zed", None, &["zed"], &[]
    ),
    app!(
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
    app!(
        "gh_cli", "GitHub CLI (gh)", "GitHub", "cli.github.com",
        "Official GitHub command-line tool for PRs & releases", "CLI oficial do GitHub para PRs e releases",
        AppCategory::Development, "GitHub.cli", None, &["github cli"], &[PresetKind::Developer]
    ),
    app!(
        "github_desktop", "GitHub Desktop", "GitHub", "github.com",
        "Visual Git client for repositories and PRs", "Cliente visual intuitivo para repositórios GitHub",
        AppCategory::Development, "GitHub.GitHubDesktop",
        Some(DirectSource::StaticUrl {
            url: "https://central.github.com/deployments/desktop/desktop/latest/win32",
            kind: InstallerKind::Exe, silent_args: &["-s"],
        }),
        &["github desktop"], &[]
    ),
    app!(
        "windows_terminal", "Windows Terminal", "Microsoft", "microsoft.com",
        "GPU-accelerated modern tabbed terminal", "Terminal moderno com abas e aceleração GPU",
        AppCategory::Development, "Microsoft.WindowsTerminal", None, &["windows terminal"], &[PresetKind::Developer]
    ),
    app!(
        "powershell7", "PowerShell 7", "Microsoft", "microsoft.com",
        "Modern cross-platform PowerShell shell", "Shell PowerShell 7 moderno e rápido",
        AppCategory::Development, "Microsoft.PowerShell", None, &["powershell 7"], &[]
    ),
    app!(
        "nodejs", "Node.js LTS", "OpenJS Foundation", "nodejs.org",
        "JavaScript runtime built on Chrome V8", "Runtime JavaScript LTS com npm incluso",
        AppCategory::Development, "OpenJS.NodeJS.LTS", None, &["node.js"], &[PresetKind::Developer]
    ),
    app!(
        "bun", "Bun", "Oven", "bun.sh",
        "JavaScript/TypeScript runtime and package manager", "Runtime e gerenciador de pacotes JS/TS",
        AppCategory::Development, "Oven-sh.Bun", None, &["bun"], &[]
    ),
    app!(
        "python3", "Python 3.12", "Python Software Foundation", "python.org",
        "Scripting, backend and data language", "Linguagem Python 3 com pip incluso",
        AppCategory::Development, "Python.Python.3.12", None, &["python 3."], &[PresetKind::Developer]
    ),
    app!(
        "golang", "Go", "Google", "go.dev",
        "Fast statically typed systems & backend language", "Linguagem compilada rápida para backend e CLI",
        AppCategory::Development, "GoLang.Go", None, &["go programming language"], &[]
    ),
    app!(
        "rustup", "Rustup (Rust)", "Rust Project", "rust-lang.org",
        "Official Rust compiler & Cargo toolchain", "Instalador oficial da toolchain Rust e Cargo",
        AppCategory::Development, "Rustlang.Rustup",
        Some(DirectSource::StaticUrl {
            url: "https://win.rustup.rs/x86_64",
            kind: InstallerKind::Exe, silent_args: &["-y"],
        }),
        &["rustup"], &[]
    ),
    app!(
        "docker", "Docker Desktop", "Docker Inc.", "docker.com",
        "Containers, Compose and local Kubernetes", "Ambiente de containers e Docker Compose",
        AppCategory::Development, "Docker.DockerDesktop", None, &["docker desktop"], &[]
    ),
    app!(
        "neovim", "Neovim", "Neovim", "neovim.io",
        "Extensible Vim-based modal editor", "Editor modal extensível baseado em Vim",
        AppCategory::Development, "Neovim.Neovim",
        Some(DirectSource::GitHubRelease {
            repo: "neovim/neovim", asset_contains: &["win64"],
            asset_excludes: &[], asset_ends_with: ".msi",
            kind: InstallerKind::Msi, silent_args: &["/qn", "/norestart"],
        }),
        &["neovim"], &[]
    ),
    app!(
        "postman", "Postman", "Postman Inc.", "postman.com",
        "REST & GraphQL API platform", "Plataforma completa para testar APIs",
        AppCategory::Development, "Postman.Postman", None, &["postman"], &[]
    ),
    app!(
        "insomnia", "Insomnia", "Kong", "insomnia.rest",
        "Lightweight open-source REST & GraphQL client", "Cliente leve para APIs REST e GraphQL",
        AppCategory::Development, "Insomnia.Insomnia", None, &["insomnia"], &[]
    ),
    app!(
        "dbeaver", "DBeaver Community", "DBeaver", "dbeaver.io",
        "Universal SQL database GUI client", "Cliente universal para bancos SQL e NoSQL",
        AppCategory::Development, "dbeaver.dbeaver", None, &["dbeaver"], &[]
    ),
    app!(
        "jetbrains_toolbox", "JetBrains Toolbox", "JetBrains", "jetbrains.com",
        "Manage IntelliJ, PyCharm, RustRover, Rider & WebStorm", "Gerenciador oficial de todas as IDEs JetBrains",
        AppCategory::Development, "JetBrains.Toolbox", None, &["jetbrains toolbox"], &[]
    ),
    app!(
        "intellij_idea", "IntelliJ IDEA", "JetBrains", "jetbrains.com",
        "Java, Kotlin and JVM IDE by JetBrains", "IDE da JetBrains para Java, Kotlin e JVM",
        AppCategory::Development, "JetBrains.IntelliJIDEA", None, &["intellij idea"], &[PresetKind::Developer]
    ),
    app!(
        "intellij_community", "IntelliJ IDEA Community", "JetBrains", "jetbrains.com",
        "Free open-source Java & Kotlin IDE by JetBrains", "Versão gratuita da IDE IntelliJ IDEA para Java e Kotlin",
        AppCategory::Development, "JetBrains.IntelliJIDEA.Community", None, &["intellij idea community"], &[]
    ),
    app!(
        "pycharm", "PyCharm", "JetBrains", "jetbrains.com",
        "Professional Python, data science & web IDE", "IDE profissional JetBrains para Python e ciência de dados",
        AppCategory::Development, "JetBrains.PyCharm", None, &["pycharm"], &[]
    ),
    app!(
        "webstorm", "WebStorm", "JetBrains", "jetbrains.com",
        "Smart JavaScript, TypeScript, React & Next.js IDE", "IDE inteligente JetBrains para JS, TypeScript e React",
        AppCategory::Development, "JetBrains.WebStorm", None, &["webstorm"], &[]
    ),
    app!(
        "clion", "CLion", "JetBrains", "jetbrains.com",
        "Cross-platform C & C++ IDE with CMake & debugger", "IDE profissional JetBrains para C e C++ com CMake",
        AppCategory::Development, "JetBrains.CLion", None, &["clion"], &[]
    ),
    app!(
        "rider", "JetBrains Rider", "JetBrains", "jetbrains.com",
        "Fast .NET, C#, Unity & Unreal Engine IDE", "IDE rápida para .NET, C#, Unity e Unreal Engine",
        AppCategory::Development, "JetBrains.Rider", None, &["jetbrains rider"], &[]
    ),
    app!(
        "datagrip", "DataGrip", "JetBrains", "jetbrains.com",
        "Powerful multi-engine SQL database IDE", "IDE poderosa para bancos de dados PostgreSQL, MySQL e SQL",
        AppCategory::Development, "JetBrains.DataGrip", None, &["datagrip"], &[]
    ),
    app!(
        "rustrover", "RustRover", "JetBrains", "jetbrains.com",
        "JetBrains IDE for Rust", "IDE da JetBrains para Rust",
        AppCategory::Development, "JetBrains.RustRover", None, &["rustrover"], &[]
    ),
    app!(
        "goland", "GoLand", "JetBrains", "jetbrains.com",
        "JetBrains IDE for Go", "IDE da JetBrains para Go",
        AppCategory::Development, "JetBrains.GoLand", None, &["goland"], &[]
    ),
    app!(
        "phpstorm", "PhpStorm", "JetBrains", "jetbrains.com",
        "JetBrains IDE for PHP and web", "IDE da JetBrains para PHP e web",
        AppCategory::Development, "JetBrains.PhpStorm", None, &["phpstorm"], &[]
    ),
    app!(
        "rubymine", "RubyMine", "JetBrains", "jetbrains.com",
        "JetBrains IDE for Ruby and Rails", "IDE da JetBrains para Ruby e Rails",
        AppCategory::Development, "JetBrains.RubyMine", None, &["rubymine"], &[]
    ),
    app!(
        "jb_fleet", "JetBrains Fleet", "JetBrains", "jetbrains.com",
        "Lightweight multi-language editor by JetBrains", "Editor leve multi-linguagem da JetBrains",
        AppCategory::Development, "JetBrains.FleetLauncher.Preview", None, &["fleet"], &[]
    ),
    app!(
        "jb_dotpeek", "JetBrains dotPeek", "JetBrains", "jetbrains.com",
        "Free .NET assembly browser and decompiler", "Navegador e descompilador gratuito de assemblies .NET",
        AppCategory::Development, "JetBrains.dotPeek", None, &["dotpeek"], &[]
    ),
    app!(
        "jb_resharper", "JetBrains ReSharper", "JetBrains", "jetbrains.com",
        "Code analysis and refactoring extension for Visual Studio", "Extensão de produtividade e refatoração para Visual Studio",
        AppCategory::Development, "JetBrains.ReSharper", None, &["resharper"], &[]
    ),
    app!(
        "jb_dottrace", "JetBrains dotTrace", "JetBrains", "jetbrains.com",
        ".NET performance profiler by JetBrains", "Profiler de desempenho .NET da JetBrains",
        AppCategory::Development, "JetBrains.dotTrace", None, &["dottrace"], &[]
    ),
    app!(
        "jb_aqua", "JetBrains Aqua", "JetBrains", "jetbrains.com",
        "IDE for test automation", "IDE da JetBrains para automação de testes",
        AppCategory::Development, "JetBrains.Aqua.Preview", None, &["jetbrains aqua"], &[]
    ),
    app!(
        "android_studio", "Android Studio", "Google", "developer.android.com",
        "Official Google IDE & emulator for Android apps", "IDE oficial do Google com emulador para apps Android",
        AppCategory::Development, "Google.AndroidStudio", None, &["android studio"], &[]
    ),
    app!(
        "deno", "Deno", "Deno Land", "deno.com",
        "Secure modern JavaScript & TypeScript runtime", "Runtime moderno e seguro para JavaScript e TypeScript",
        AppCategory::Development, "DenoLand.Deno", None, &["deno"], &[]
    ),
    app!(
        "uv_python", "uv (Python Manager)", "Astral", "astral.sh",
        "Python package and environment manager in Rust", "Gerenciador de pacotes e ambientes Python em Rust",
        AppCategory::Development, "astral-sh.uv", None, &["uv"], &[]
    ),
    app!(
        "bruno", "Bruno API Client", "Bruno", "usebruno.com",
        "Fast Git-friendly open-source Postman alternative", "Alternativa open-source local e rápida ao Postman",
        AppCategory::Development, "Bruno.Bruno", None, &["bruno"], &[]
    ),
    app!(
        "gitkraken", "GitKraken", "Axosoft", "gitkraken.com",
        "Visual Git commit graph, merge tool & workflows", "Cliente Git visual com grafo de commits e merge",
        AppCategory::Development, "Axosoft.GitKraken", None, &["gitkraken"], &[]
    ),
    app!(
        "unity_hub", "Unity Hub", "Unity Technologies", "unity.com",
        "Manage Unity Engine versions & 2D/3D game projects", "Gerencie versões da Unity Engine e projetos 2D/3D",
        AppCategory::Development, "Unity.UnityHub", None, &["unity hub"], &[]
    ),
    app!(
        "godot", "Godot Engine", "Godot Foundation", "godotengine.org",
        "Lightweight open-source 2D & 3D game engine", "Engine open-source leve para jogos 2D e 3D",
        AppCategory::Development, "GodotEngine.GodotEngine", None, &["godot"], &[]
    ),
    app!(
        "wireshark", "Wireshark", "Wireshark Foundation", "wireshark.org",
        "Network protocol and packet analyzer", "Analisador de protocolos e pacotes de rede",
        AppCategory::Development, "WiresharkFoundation.Wireshark", None, &["wireshark"], &[]
    ),
    app!(
        "winscp", "WinSCP", "Martin Prikryl", "winscp.net",
        "Fast SFTP, SCP and FTP file client", "Cliente rápido SFTP, SCP e FTP",
        AppCategory::Development, "WinSCP.WinSCP", None, &["winscp"], &[]
    ),

    // Media & creative
    app!(
        "vlc", "VLC Media Player", "VideoLAN", "videolan.org",
        "Universal open-source audio & video player", "Reprodutor universal open-source para qualquer mídia",
        AppCategory::Media, "VideoLAN.VLC", None, &["vlc media player"], &[PresetKind::Essentials, PresetKind::Creator]
    ),
    app!(
        "mpv", "mpv Media Player", "mpv", "mpv.io",
        "Minimalist GPU-accelerated video player", "Reprodutor de vídeo minimalista acelerado por GPU",
        AppCategory::Media, "shinchiro.mpv", None, &["mpv"], &[]
    ),
    app!(
        "obs", "OBS Studio", "OBS Project", "obsproject.com",
        "Screen recording and live streaming", "Gravação de tela e transmissão ao vivo",
        AppCategory::Media, "OBSProject.OBSStudio",
        Some(DirectSource::GitHubRelease {
            repo: "obsproject/obs-studio", asset_contains: &["Windows-x64-Installer"],
            asset_excludes: &["arm64"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["obs studio"], &[PresetKind::Gaming, PresetKind::Creator]
    ),
    app!(
        "spotify", "Spotify", "Spotify AB", "spotify.com",
        "Music and podcast streaming client", "Streaming de músicas e podcasts",
        AppCategory::Media, "Spotify.Spotify",
        Some(DirectSource::StaticUrl {
            url: "https://download.scdn.co/SpotifySetup.exe",
            kind: InstallerKind::Exe, silent_args: &["/silent"],
        }),
        &["spotify"], &[PresetKind::Gaming]
    ),
    app!(
        "capcut", "CapCut", "ByteDance", "capcut.com",
        "Fast desktop video editor with effects & captions", "Editor de vídeo rápido com efeitos e legendas",
        AppCategory::Media, "ByteDance.CapCut", None, &["capcut"], &[]
    ),
    app!(
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
    app!(
        "flameshot", "Flameshot", "Flameshot", "flameshot.org",
        "Clean annotated screenshot capture utility", "Captura de tela com anotações rápidas",
        AppCategory::Media, "Flameshot.Flameshot", None, &["flameshot"], &[]
    ),
    app!(
        "lightshot", "Lightshot", "Skillbrains", "app.prntscr.com",
        "Lightweight instant region screenshot tool", "Captura de tela instantânea e levíssima",
        AppCategory::Media, "Skillbrains.Lightshot", None, &["lightshot"], &[]
    ),
    app!(
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
    app!(
        "gimp", "GIMP", "GIMP Team", "gimp.org",
        "Open-source raster graphics editor", "Editor de imagens open-source completo",
        AppCategory::Media, "GIMP.GIMP.3", None, &["gimp"], &[PresetKind::Creator]
    ),
    app!(
        "krita", "Krita", "KDE", "krita.org",
        "Digital painting and illustration studio", "Pintura digital e ilustração profissional",
        AppCategory::Media, "KDE.Krita", None, &["krita"], &[]
    ),
    app!(
        "inkscape", "Inkscape", "Inkscape", "inkscape.org",
        "Open-source SVG vector graphics editor", "Editor vetorial SVG open-source",
        AppCategory::Media, "Inkscape.Inkscape", None, &["inkscape"], &[]
    ),
    app!(
        "figma", "Figma", "Figma Inc.", "figma.com",
        "Collaborative UI design and prototyping desktop app", "Design de interfaces e prototipagem",
        AppCategory::Media, "Figma.Figma", None, &["figma"], &[]
    ),
    app!(
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
    app!(
        "blender", "Blender", "Blender Foundation", "blender.org",
        "3D modeling, animation, rendering & VFX", "Suíte 3D completa para modelagem e VFX",
        AppCategory::Media, "BlenderFoundation.Blender", None, &["blender"], &[PresetKind::Creator]
    ),
    app!(
        "paint_net", "Paint.NET", "dotPDN", "getpaint.net",
        "Fast, lightweight everyday image editor", "Editor de imagens leve e super rápido",
        AppCategory::Media, "dotPDN.PaintDotNet", None, &["paint.net"], &[]
    ),
    app!(
        "ffmpeg", "FFmpeg", "Gyan / FFmpeg", "ffmpeg.org",
        "Essential CLI audio/video encoder & processor", "Codificador CLI universal de vídeo e áudio",
        AppCategory::Media, "Gyan.FFmpeg", None, &["ffmpeg"], &[]
    ),
    app!(
        "fxsound", "FxSound", "FxSound", "fxsound.com",
        "Free real-time audio booster & EQ presets", "Melhorador de áudio e equalizador em tempo real",
        AppCategory::Media, "FxSound.FxSound", None, &["fxsound"], &[]
    ),
    app!(
        "kdenlive", "Kdenlive", "KDE", "kdenlive.org",
        "Free open-source multi-track desktop video editor", "Editor de vídeo multipista gratuito e open-source",
        AppCategory::Media, "KDE.Kdenlive", None, &["kdenlive"], &[]
    ),
    app!(
        "shotcut", "Shotcut", "Meltytech", "shotcut.org",
        "Lightweight cross-platform 4K video editor", "Editor de vídeo 4K leve e open-source",
        AppCategory::Media, "Meltytech.Shotcut", None, &["shotcut"], &[]
    ),
    app!(
        "darktable", "darktable", "darktable", "darktable.org",
        "Open-source RAW photography editor (Lightroom alt)", "Editor fotográfico RAW open-source (alternativa ao Lightroom)",
        AppCategory::Media, "darktable.darktable", None, &["darktable"], &[]
    ),
    app!(
        "rawtherapee", "RawTherapee", "RawTherapee", "rawtherapee.com",
        "Advanced non-destructive RAW image processing", "Processamento fotográfico RAW avançado sem perdas",
        AppCategory::Media, "RawTherapee.RawTherapee", None, &["rawtherapee"], &[]
    ),
    app!(
        "screentogif", "ScreenToGif", "Nicke Manarin", "screentogif.com",
        "Record screen & edit lightweight GIFs / WebP clips", "Grave a tela e edite GIFs e clipes levíssimos",
        AppCategory::Media, "NickeManarin.ScreenToGif", None, &["screentogif"], &[]
    ),
    app!(
        "potplayer", "PotPlayer", "Kakao / Daum", "potplayer.daum.net",
        "Feature-rich hardware-accelerated 4K/HDR media player", "Player de vídeo 4K/HDR completo acelerado por GPU",
        AppCategory::Media, "Daum.PotPlayer", None, &["potplayer"], &[]
    ),
    app!(
        "reaper", "REAPER DAW", "Cockos", "reaper.fm",
        "Lightweight digital audio workstation", "DAW leve para produção e edição de áudio",
        AppCategory::Media, "Cockos.REAPER", None, &["reaper"], &[]
    ),

    // Utilities
    app!(
        "7zip", "7-Zip", "Igor Pavlov", "7-zip.org",
        "High-compression file archiver", "Compactador de arquivos de alta compressão",
        AppCategory::Utilities, "7zip.7zip",
        Some(DirectSource::GitHubRelease {
            repo: "ip7z/7zip", asset_contains: &["-x64"],
            asset_excludes: &["arm64", "extra"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["7-zip"], &[PresetKind::Essentials, PresetKind::Developer, PresetKind::Gaming, PresetKind::Creator]
    ),
    app!(
        "nanazip", "NanaZip", "M2-Team", "github.com",
        "Modern 7-Zip fork for Windows 11 context menu", "Fork moderno do 7-Zip com menu nativo do Win11",
        AppCategory::Utilities, "M2Team.NanaZip", None, &["nanazip"], &[]
    ),
    app!(
        "winrar", "WinRAR", "RARLAB", "win-rar.com",
        "Classic RAR & ZIP archive manager", "Compactador clássico RAR e ZIP",
        AppCategory::Utilities, "RARLab.WinRAR", None, &["winrar"], &[]
    ),
    app!(
        "everything", "Everything", "voidtools", "voidtools.com",
        "Instant NTFS search across millions of files", "Busca instantânea de arquivos em milissegundos",
        AppCategory::Utilities, "voidtools.Everything",
        Some(DirectSource::StaticUrl {
            url: "https://www.voidtools.com/Everything-1.4.1.1026.x64-Setup.exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["everything"], &[PresetKind::Essentials, PresetKind::Developer, PresetKind::Gaming, PresetKind::Creator]
    ),
    app!(
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
    app!(
        "wiztree", "WizTree", "Antibody Software", "diskanalyzer.com",
        "Fastest NTFS disk space analyzer", "Analisador de espaço em disco mais rápido",
        AppCategory::Utilities, "AntibodySoftware.WizTree", None, &["wiztree"], &[PresetKind::Essentials]
    ),
    app!(
        "windhawk", "Windhawk", "Ramen Software", "windhawk.net",
        "Modular Windows 11/10 customization marketplace", "Mods leves para customizar barra e Explorer no Windows",
        AppCategory::Utilities, "RamenSoftware.Windhawk", None, &["windhawk"], &[]
    ),
    app!(
        "flow_launcher", "Flow Launcher", "Flow Launcher", "flowlauncher.com",
        "Keyboard launcher for files, apps and web", "Lançador rápido por teclado para arquivos e apps",
        AppCategory::Utilities, "Flow-Launcher.Flow-Launcher", None, &["flow launcher"], &[]
    ),
    app!(
        "translucenttb", "TranslucentTB", "Charles Milette", "github.com",
        "Lightweight translucent/clear Windows taskbar", "Barra de tarefas transparente levíssima",
        AppCategory::Utilities, "9PF4KZ2VN4W9", None, &["translucenttb"], &[]
    ),
    app!(
        "twinkle_tray", "Twinkle Tray", "xanderfrangos", "twinkletray.com",
        "Control external monitor DDC/CI brightness from tray", "Controla brilho real de monitores externos pela bandeja",
        AppCategory::Utilities, "xanderfrangos.twinkletray", None, &["twinkle tray"], &[]
    ),
    app!(
        "ditto", "Ditto Clipboard", "Scott Brogden", "ditto-cp.sourceforge.io",
        "Fast searchable clipboard history manager", "Histórico de área de transferência rápido e pesquisável",
        AppCategory::Utilities, "Ditto.Ditto", None, &["ditto"], &[]
    ),
    app!(
        "notepadpp", "Notepad++", "Don Ho", "notepad-plus-plus.org",
        "Lightweight text and source code editor", "Editor leve de texto e código-fonte",
        AppCategory::Utilities, "Notepad++.Notepad++",
        Some(DirectSource::GitHubRelease {
            repo: "notepad-plus-plus/notepad-plus-plus", asset_contains: &["Installer.x64"],
            asset_excludes: &["arm64"], asset_ends_with: ".exe",
            kind: InstallerKind::Exe, silent_args: &["/S"],
        }),
        &["notepad++"], &[]
    ),
    app!(
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
    app!(
        "qbittorrent", "qBittorrent", "qBittorrent Project", "qbittorrent.org",
        "Clean ad-free open-source BitTorrent client", "Cliente BitTorrent open-source limpo",
        AppCategory::Utilities, "qBittorrent.qBittorrent", None, &["qbittorrent"], &[]
    ),
    app!(
        "jdownloader", "JDownloader 2", "AppWork", "jdownloader.org",
        "High-speed multi-connection download manager", "Gerenciador de downloads multi-conexão",
        AppCategory::Utilities, "AppWork.JDownloader", None, &["jdownloader"], &[]
    ),
    app!(
        "rufus", "Rufus", "Pete Batard", "rufus.ie",
        "Create bootable Windows & Linux USB drives", "Cria pendrives bootáveis do Windows e Linux",
        AppCategory::Utilities, "Rufus.Rufus", None, &["rufus"], &[]
    ),
    app!(
        "eartrumpet", "EarTrumpet", "File-New-Project", "eartrumpet.app",
        "Per-app volume control in the taskbar", "Mixer de volume individual por app na barra",
        AppCategory::Utilities, "File-New-Project.EarTrumpet", None, &["eartrumpet"], &[PresetKind::Gaming]
    ),
    app!(
        "tailscale", "Tailscale", "Tailscale Inc.", "tailscale.com",
        "Zero-config WireGuard mesh VPN", "Rede privada WireGuard instantânea",
        AppCategory::Utilities, "Tailscale.Tailscale", None, &["tailscale"], &[]
    ),
    app!(
        "cloudflare_warp", "Cloudflare WARP", "Cloudflare", "one.one.one.one",
        "Fast encrypted 1.1.1.1 DNS & tunnel", "DNS criptografado 1.1.1.1 rápido da Cloudflare",
        AppCategory::Utilities, "Cloudflare.Warp", None, &["cloudflare warp", "cloudflare one"], &[]
    ),
    app!(
        "proton_vpn", "Proton VPN", "Proton AG", "protonvpn.com",
        "Privacy-focused audited VPN with free tier", "VPN privada auditada com plano gratuito",
        AppCategory::Utilities, "Proton.ProtonVPN", None, &["proton vpn", "protonvpn"], &[]
    ),
    app!(
        "files_app", "Files (Modern Explorer)", "Files Community", "files.community",
        "Modern tabbed Windows file manager with preview & tags", "Explorador de arquivos moderno com abas, tags e prévia",
        AppCategory::Utilities, "FilesCommunity.Files", None, &["files - stable", "files app"], &[]
    ),
    app!(
        "onecommander", "OneCommander", "Milos Paripovic", "onecommander.com",
        "Dual-pane file manager with column view", "Gerenciador de arquivos dual-pane com colunas",
        AppCategory::Utilities, "MilosParipovic.OneCommander", None, &["onecommander"], &[]
    ),
    app!(
        "teracopy", "TeraCopy", "Code Sector", "codesector.com",
        "High-speed verified file copy & move with pause/resume", "Cópia ultrarrápida de arquivos com verificação e pausa",
        AppCategory::Utilities, "CodeSector.TeraCopy", None, &["teracopy"], &[]
    ),
    app!(
        "peazip", "PeaZip", "Giorgio Tani", "peazip.github.io",
        "Open-source 200+ format archive manager", "Gerenciador open-source para mais de 200 formatos compactados",
        AppCategory::Utilities, "Giorgiotani.Peazip", None, &["peazip"], &[]
    ),
    app!(
        "treesize_free", "TreeSize Free", "JAM Software", "jam-software.com",
        "Fast visual folder size analyzer for NTFS drives", "Analise rapidamente quais pastas ocupam mais espaço",
        AppCategory::Utilities, "JAMSoftware.TreeSize.Free", None, &["treesize"], &[]
    ),
    app!(
        "syncthing", "Syncthing", "Syncthing", "syncthing.net",
        "Private peer-to-peer continuous folder synchronization", "Sincronização P2P criptografada de pastas entre PCs",
        AppCategory::Utilities, "BillStewart.SyncthingWindowsSetup", None, &["syncthing"], &[]
    ),
    app!(
        "rclone", "Rclone", "Rclone", "rclone.org",
        "Sync & mount Google Drive, S3, OneDrive & 70+ clouds", "Sincronize e monte Google Drive, S3 e 70+ nuvens",
        AppCategory::Utilities, "Rclone.Rclone", None, &["rclone"], &[]
    ),
    app!(
        "balena_etcher", "balenaEtcher", "Balena", "etcher.balena.io",
        "Flash OS images cleanly to USB drives & SD cards", "Grave ISOs em pendrives e cartões SD com verificação",
        AppCategory::Utilities, "Balena.Etcher", None, &["balenaetcher"], &[]
    ),
    app!(
        "ventoy", "Ventoy", "Ventoy", "ventoy.net",
        "Multi-boot USB creator — drag & drop multiple ISOs", "Pendrive multi-boot: arraste várias ISOs de Windows/Linux",
        AppCategory::Utilities, "Ventoy.Ventoy", None, &["ventoy"], &[]
    ),

    // Productivity
    app!(
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
    app!(
        "notion", "Notion", "Notion Labs", "notion.so",
        "Connected docs, wikis and project workspace", "Documentos, wikis e gestão de projetos",
        AppCategory::Productivity, "Notion.Notion", None, &["notion"], &[]
    ),
    app!(
        "sumatrapdf", "SumatraPDF", "Krzysztof Kowalczyk", "sumatrapdfreader.org",
        "Instant-loading minimal PDF and ePub reader", "Leitor de PDF e ePub minimalista instantâneo",
        AppCategory::Productivity, "SumatraPDF.SumatraPDF", None, &["sumatrapdf"], &[PresetKind::Essentials]
    ),
    app!(
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
    app!(
        "keepassxc", "KeePassXC", "KeePassXC Team", "keepassxc.org",
        "Offline encrypted local password vault", "Cofre de senhas local criptografado offline",
        AppCategory::Productivity, "KeePassXCTeam.KeePassXC", None, &["keepassxc"], &[]
    ),
    app!(
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
    app!(
        "anydesk", "AnyDesk", "AnyDesk", "anydesk.com",
        "Fast lightweight remote desktop access", "Acesso remoto leve e rápido",
        AppCategory::Productivity, "AnyDeskSoftwareGmbH.AnyDesk", None, &["anydesk"], &[]
    ),
    app!(
        "libreoffice", "LibreOffice", "The Document Foundation", "libreoffice.org",
        "Free open-source office document suite", "Suíte de escritório gratuita e open-source",
        AppCategory::Productivity, "TheDocumentFoundation.LibreOffice", None, &["libreoffice"], &[]
    ),
    app!(
        "onlyoffice", "ONLYOFFICE Desktop", "Ascensio System", "onlyoffice.com",
        "Modern Microsoft Office-compatible suite", "Suíte moderna altamente compatível com MS Office",
        AppCategory::Productivity, "ONLYOFFICE.DesktopEditors", None, &["onlyoffice"], &[]
    ),
    app!(
        "anki", "Anki", "Anki", "apps.ankiweb.net",
        "Intelligent spaced-repetition flashcards for studying", "Flashcards com repetição espaçada para estudos e concursos",
        AppCategory::Productivity, "Anki.Anki", None, &["anki"], &[]
    ),
    app!(
        "zotero", "Zotero", "Digital Scholar", "zotero.org",
        "Academic research PDF library & citation manager", "Organizador de artigos científicos, PDFs e citações ABNT",
        AppCategory::Productivity, "DigitalScholar.Zotero", None, &["zotero"], &[]
    ),
    app!(
        "logseq", "Logseq", "Logseq", "logseq.com",
        "Local-first outliner & knowledge graph for study notes", "Notas em tópicos e grafo de conhecimento local-first",
        AppCategory::Productivity, "Logseq.Logseq", None, &["logseq"], &[]
    ),
    app!(
        "calibre", "Calibre", "Kovid Goyal", "calibre-ebook.com",
        "Complete e-book library manager, viewer & converter", "Biblioteca, leitor e conversor completo de e-books e Kindle",
        AppCategory::Productivity, "calibre.calibre", None, &["calibre"], &[]
    ),
    app!(
        "miktex", "MiKTeX (LaTeX)", "MiKTeX", "miktex.org",
        "Modern LaTeX distribution for academic papers & theses", "Distribuição LaTeX para artigos científicos e TCC",
        AppCategory::Productivity, "MiKTeX.MiKTeX", None, &["miktex"], &[]
    ),
    app!(
        "geogebra", "GeoGebra Calculator Suite", "GeoGebra", "geogebra.org",
        "Interactive graphing, 3D math, algebra & geometry", "Calculadora gráfica 3D interativa para matemática e física",
        AppCategory::Productivity, "GeoGebra.CalculatorSuite", None, &["geogebra"], &[]
    ),
    app!(
        "pdf24", "PDF24 Creator", "Geek Software", "pdf24.org",
        "Offline merge, compress, sign & convert PDF toolkit", "Junte, comprima, assine e converta PDFs 100% offline",
        AppCategory::Productivity, "geeksoftwareGmbH.PDF24Creator", None, &["pdf24"], &[]
    ),
    app!(
        "onepassword", "1Password", "AgileBits", "1password.com",
        "Polished cross-platform password & passkey manager", "Gerenciador premium de senhas e passkeys multiplataforma",
        AppCategory::Productivity, "AgileBits.1Password", None, &["1password"], &[]
    ),
    app!(
        "proton_pass", "Proton Pass", "Proton AG", "proton.me",
        "End-to-end encrypted password & email alias vault", "Cofre de senhas criptografado com aliases de e-mail",
        AppCategory::Productivity, "Proton.ProtonPass", None, &["proton pass"], &[]
    ),
    app!(
        "todoist", "Todoist", "Doist", "todoist.com",
        "Clean fast task manager & daily planner", "Organizador de tarefas e planejamento diário limpo",
        AppCategory::Productivity, "Doist.Todoist", None, &["todoist"], &[]
    ),
    app!(
        "ticktick", "TickTick", "Appest", "ticktick.com",
        "Tasks, calendar & Pomodoro focus timer in one app", "Tarefas, calendário e timer Pomodoro para foco nos estudos",
        AppCategory::Productivity, "Appest.TickTick", None, &["ticktick"], &[]
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
        assert!(catalog().len() >= 200);
        for required in [
            "chrome",
            "discord",
            "steam",
            "vscode",
            "vs2022_community",
            "ivy_iii",
            "retoolkit",
            "ldplayer",
            "eac_eos",
            "riot_client",
            "amd_adrenalin",
            "amd_ryzen_master",
            "goland",
            "phpstorm",
            "rubymine",
            "jb_fleet",
            "jb_dotpeek",
            "jb_resharper",
            "jb_dottrace",
            "jb_aqua",
        ] {
            assert!(ids.contains(required), "Missing expected catalog id: {required}");
        }
    }
}
