fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set("ProductName", "Reforge");
        res.set("FileDescription", "Reforge — Portable Windows Post-Format Toolkit");
        res.set("LegalCopyright", "Copyright (c) 2026 Reforge Contributors (MIT)");
        if std::path::Path::new("assets/reforge.ico").exists() {
            res.set_icon("assets/reforge.ico");
        }

        // Embed requireAdministrator in release builds so Reforge.exe automatically requests UAC elevation.
        // Dev/test builds stay asInvoker so `cargo test` runs seamlessly without UAC prompts.
        let level = if std::env::var("PROFILE").as_deref() == Ok("release") {
            "requireAdministrator"
        } else {
            "asInvoker"
        };

        let manifest = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity version="1.0.0.0" processorArchitecture="*" name="Reforge" type="win32"/>
  <description>Reforge — Portable Windows Post-Format Toolkit</description>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="{level}" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}}"/>
      <supportedOS Id="{{1f676c76-80e1-4239-95bb-83d0f6d0da78}}"/>
      <supportedOS Id="{{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}}"/>
      <supportedOS Id="{{35138b9a-5d96-4fbd-8e2d-a2440225f93a}}"/>
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
      <longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware>
    </windowsSettings>
  </application>
</assembly>"#
        );

        res.set_manifest(&manifest);
        let _ = res.compile();
    }
}
