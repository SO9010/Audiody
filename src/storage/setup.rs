use std::fs;
use std::path::PathBuf;

fn config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Get the config directory path
    // Lin: Some(/home/alice/.config)
    // Win: Some(C:\Users\Alice\AppData\Roaming)
    // Mac: Some(/Users/Alice/Library/Application Support)

    let base_dir = dirs::config_dir().ok_or("Unable to get config directory")?;
    
    // Specify the name of the directory you want to create
    let new_dir = base_dir.join("Audiody");
    if new_dir.exists() {
        println!("Directory already exists!");
        return Ok(new_dir.clone());
    }

    // Create the directory
    fs::create_dir_all(new_dir.clone())?;

    println!("Directory created successfully!");

    Ok(new_dir)
}


fn music_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Get the config directory path
    // Lin: Some(/home/alice/Music)
    // Win: Some(C:\Users\Alice\Music)
    // Mac: Some(/Users/Alice/Music)

    let base_dir = dirs::audio_dir().ok_or("Unable to get audio directory")?;
    
    // Specify the name of the directory you want to create
    let new_dir = base_dir.join("Audiody");
    if new_dir.exists() {
        println!("Directory already exists!");
        return Ok(new_dir.clone());
    }

    // Create the directory
    fs::create_dir_all(new_dir.clone())?;

    println!("Directory created successfully!");

    Ok(new_dir)
}

