#[cfg(windows)]
fn main() -> std::io::Result<()> {
    use std::env;
    use std::path::Path;
    use std::path::PathBuf;

    let png_path = Path::new("assests/icons/oxide_mongo_256x256.png");
    println!("cargo:rerun-if-changed={}", png_path.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set"));
    let ico_path = out_dir.join("oxide_mongo.ico");
    write_png_as_ico(png_path, &ico_path)?;

    let mut res = winres::WindowsResource::new();
    res.set_icon(ico_path.to_str().expect("invalid icon path"));
    res.compile().expect("failed to compile Windows resources");

    Ok(())
}

#[cfg(not(windows))]
fn main() {}

#[cfg(windows)]
fn write_png_as_ico(png_path: &std::path::Path, ico_path: &std::path::Path) -> std::io::Result<()> {
    let png_data = std::fs::read(png_path)?;
    let mut ico_data = Vec::with_capacity(6 + 16 + png_data.len());

    ico_data.extend_from_slice(&0u16.to_le_bytes());
    ico_data.extend_from_slice(&1u16.to_le_bytes());
    ico_data.extend_from_slice(&1u16.to_le_bytes());

    ico_data.push(0);
    ico_data.push(0);
    ico_data.push(0);
    ico_data.push(0);
    ico_data.extend_from_slice(&1u16.to_le_bytes());
    ico_data.extend_from_slice(&32u16.to_le_bytes());
    ico_data.extend_from_slice(&(png_data.len() as u32).to_le_bytes());
    ico_data.extend_from_slice(&(6u32 + 16u32).to_le_bytes());

    ico_data.extend_from_slice(&png_data);
    std::fs::write(ico_path, ico_data)?;
    Ok(())
}
