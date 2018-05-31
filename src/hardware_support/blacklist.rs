use std::fs::File;
use std::io::{self, Read, Write};
use super::Module;

const POWER: &str = "/etc/modprobe.d/system76-power.conf";

static BLACKLIST_NVIDIA: &[u8] = br#"# Automatically generated by distinst
blacklist nouveau
blacklist nvidia
blacklist nvidia-drm
blacklist nvidia-modeset
alias nouveau off
alias nvidia off
alias nvidia-drm off
alias nvidia-modeset off
"#;

/// Disables external graphics if switchable graphics is supported.
pub fn disable_external_graphics() -> io::Result<()> {
    if let Ok(modules) = Module::all() {
        let product_name = &*product_name();
        let disable_nvidia = has_switchable_graphics(product_name)
            && modules.iter().any(|x| &x.name == "nvidia" || &x.name == "nouveau");
            
        if disable_nvidia {
            info!("libdistinst: disabling external NVIDIA graphics by default");
            File::open(POWER).and_then(|mut file| file.write_all(BLACKLIST_NVIDIA))?;
        }
    }

    Ok(())
}

/// Products which support switchable graphics.
static SWITCHABLE_GRAPHICS: &[&str] = &["oryp4", "oryp4-b"];


fn has_switchable_graphics(product: &str) -> bool {
    SWITCHABLE_GRAPHICS.contains(&product)
}

/// Path where the product name can be obtained from the DMI.
const DMI_PATH_PRODUCT_NAME: &str = "/sys/class/dmi/id/product_name";

fn product_name() -> String {
    let mut output = String::new();
    if let Ok(mut file) = File::open(DMI_PATH_PRODUCT_NAME) {
        let _ = file.read_to_string(&mut output);
        output = output.trim().into();
    }
    output
}